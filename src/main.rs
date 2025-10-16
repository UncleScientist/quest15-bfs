use std::collections::{HashMap, HashSet, VecDeque};

use iced::{
    Color, Element, Length, Point, Renderer, Size, Subscription, Task, Theme, color, mouse,
    widget::{
        Canvas,
        canvas::{Cache, Program},
    },
};

use crate::garden::Garden;

mod garden;

#[derive(Debug, PartialEq)]
enum FindPhase {
    Neighbors,
    Path,
    Done,
}

fn main() {
    let _ = iced::application("BFS Viz", GardenDisplay::update, GardenDisplay::view)
        .subscription(GardenDisplay::subscription)
        .run();
}

#[derive(Debug)]
enum SearchResult {
    Found(usize),
    Continue,
    Finished,
}

#[derive(Debug)]
enum Message {
    Tick,
}

type Position = (i64, i64);
type SearchState = (u64, Position);

#[derive(Debug)]
struct GardenDisplay {
    cache: Cache,
    garden: Garden,
    running: FindPhase,
    iterations: usize,
    queue: VecDeque<(usize, SearchState)>,
    state_visited: HashSet<SearchState>,
    loc_visited: HashSet<Position>,
    display: Vec<Vec<Color>>,
    starting_points: Vec<(char, Position)>,
    full_search: bool,
    current_type: char,
    current_start: Position,
    neighbor_list: HashMap<Position, Vec<(usize, Position)>>, // dist & position
}

impl Default for GardenDisplay {
    fn default() -> Self {
        let full_search = std::env::var_os("FULL").is_some();

        let args = std::env::args().collect::<Vec<_>>();
        if args.len() != 2 {
            println!("specify which number");
            std::process::exit(1);
        }

        let filename = format!("input/everybody_codes_e2024_q15_p{}.txt", args[1]);
        let data = std::fs::read_to_string(filename).expect("file");
        let garden = Garden::parse(&data);
        let (rows, cols) = garden.size;

        let starting_points = garden
            .herbs
            .iter()
            .chain([(&garden.start, &'[')])
            .map(|(pos, herbtype)| (*herbtype, *pos))
            .collect();

        Self {
            cache: Cache::default(),
            garden,
            running: FindPhase::Neighbors,
            iterations: 0,
            queue: VecDeque::new(),
            state_visited: HashSet::new(),
            loc_visited: HashSet::new(),
            display: vec![vec![Color::BLACK; cols]; rows],
            starting_points,
            full_search,
            current_start: (0, 0),
            current_type: ' ',
            neighbor_list: HashMap::new(),
        }
    }
}

impl GardenDisplay {
    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Tick => {
                if self.running != FindPhase::Done {
                    self.tick();

                    let visited_color =
                        [color!(0x008000), color!(0x005000)][self.starting_points.len() % 2];
                    let unvisited_color =
                        [color!(0x005000), color!(0x008000)][self.starting_points.len() % 2];

                    self.cache.clear();

                    for loc in &self.garden.maze {
                        self.display[loc.0 as usize][loc.1 as usize] = unvisited_color;
                    }

                    for loc in &self.loc_visited {
                        self.display[loc.0 as usize][loc.1 as usize] = visited_color;
                    }

                    for (loc, value) in &self.garden.herbs {
                        let idx = (*value as u8 - b'A') as usize;
                        self.display[loc.0 as usize][loc.1 as usize] = COLOR_LIST[idx];
                    }

                    for (_, (_, loc)) in &self.queue {
                        self.display[loc.0 as usize][loc.1 as usize] = color!(0x0000a0);
                    }

                    for loc in self.neighbor_list.keys() {
                        self.display[loc.0 as usize][loc.1 as usize] = color!(0xff0000);
                    }
                }
            }
        }
        Task::none()
    }

    fn tick(&mut self) {
        match self.running {
            FindPhase::Neighbors => {
                for _ in 0..50 {
                    self.iterations += 1;
                    match if self.full_search {
                        self.step()
                    } else {
                        self.simple_step()
                    } {
                        SearchResult::Found(steps) => {
                            println!(
                                "found in {steps} steps, after {} iterations",
                                self.iterations
                            );
                            self.running = FindPhase::Done;
                        }
                        SearchResult::Continue => {}
                        SearchResult::Finished => {
                            self.running = FindPhase::Path;
                            break;
                        }
                    }
                }
            }
            FindPhase::Path => {
                if let Some(result) = aoclib::ucs(
                    &(self.garden.herb_types, self.garden.start),
                    |(remaining, pos)| {
                        let Some(neighbors) = self.neighbor_list.get(pos) else {
                            panic!("found pos {pos:?} with no neighbors");
                        };
                        neighbors
                            .iter()
                            .filter_map(|(dist, next_pos)| {
                                if *next_pos == self.garden.start {
                                    if *remaining == 0 {
                                        Some(((0, *next_pos), *dist))
                                    } else {
                                        None
                                    }
                                } else {
                                    let Some(chbit) = self.garden.herbs.get(next_pos) else {
                                        panic!("Found {next_pos:?} without a herb there");
                                    };
                                    let bit = 1u64 << (*chbit as u8 - b'A');
                                    if *remaining & bit != 0 {
                                        let next_remaining = *remaining & !bit;
                                        Some(((next_remaining, *next_pos), *dist))
                                    } else {
                                        None
                                    }
                                }
                            })
                            .collect()
                    },
                    |(remaining, pos)| *remaining == 0 && *pos == self.garden.start,
                ) {
                    println!("total distance = {}", result.1);
                } else {
                    println!("path not found");
                }
                self.running = FindPhase::Done;
            }
            FindPhase::Done => {
                unreachable!()
            }
        }
    }

    fn view(&self) -> Element<'_, Message> {
        Canvas::new(self)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }

    fn subscription(&self) -> Subscription<Message> {
        iced::time::every(std::time::Duration::from_millis(10)).map(|_| Message::Tick)
    }

    fn simple_step(&mut self) -> SearchResult {
        if self.queue.is_empty()
            && let Some((herb_type, next_pos)) = self.starting_points.pop()
        {
            // println!("next starting point {next_pos:?}");
            self.loc_visited.clear();
            self.state_visited.clear();
            self.current_type = herb_type;
            self.current_start = next_pos;
            self.queue.push_back((0, (0, next_pos)));
            self.neighbor_list.insert(self.current_start, Vec::new());
        }

        if let Some((dist, (_, pos))) = self.queue.pop_front() {
            if self.state_visited.insert((0, pos)) {
                if let Some(herb) = self.garden.herbs.get(&pos) {
                    if *herb != self.current_type {
                        self.neighbor_list
                            .entry(self.current_start)
                            .or_default()
                            .push((dist, pos));
                    }
                    // println!("Found {herb} at distance {dist} @ {pos:?}");
                } else if pos == self.garden.start && self.current_type != '[' {
                    self.neighbor_list
                        .entry(self.current_start)
                        .or_default()
                        .push((dist, pos));
                }
                self.loc_visited.insert(pos);
                for neighbor in self.garden.all_neighbors(&pos) {
                    if !self.state_visited.contains(&(0, neighbor)) {
                        self.queue.push_back((dist + 1, (0, neighbor)));
                    }
                }
            }
            //            if self.queue.is_empty() {
            //                println!(
            //                    "For herb {} at position {:?}, these are the neighbors:",
            //                    self.current_type, self.current_start
            //                );
            //                println!("{:?}", self.neighbor_list.get(&self.current_start));
            //                // std::process::exit(0);
            //            }
            SearchResult::Continue
        } else if self.starting_points.is_empty() {
            SearchResult::Finished
        } else {
            SearchResult::Continue
        }
    }

    fn step(&mut self) -> SearchResult {
        if let Some((dist, entry)) = self.queue.pop_front() {
            if self.garden.is_end(&entry) {
                return SearchResult::Found(dist);
            }
            if self.state_visited.insert(entry) {
                self.loc_visited.insert(entry.1);
                for neighbor in self.garden.neighbors(&entry) {
                    if !self.state_visited.contains(&neighbor) {
                        self.queue.push_back((dist + 1, neighbor));
                    }
                }
            }
            SearchResult::Continue
        } else {
            SearchResult::Finished
        }
    }
}

// Adapted from https://stackoverflow.com/questions/51203917/math-behind-hsv-to-rgb-conversion-of-colors
// h: 0..360, s & v: 0..100
const fn hsv_to_rgb(h: f32, s: f32, v: f32) -> Color {
    let h = h / 360.0;
    let s = s / 100.0;
    let v = v / 100.0;

    let i = (h * 6.0).floor();
    let f = h * 6.0 - i;
    let p = v * (1.0 - s);
    let q = v * (1.0 - f * s);
    let t = v * (1.0 - (1.0 - f) * s);

    let v = v * 255.0;
    let t = t * 255.0;
    let p = p * 255.0;
    let q = q * 255.0;

    match (i as u32) % 6 {
        0 => color!(v, t, p),
        1 => color!(q, v, p),
        2 => color!(p, v, t),
        3 => color!(p, q, v),
        4 => color!(t, p, v),
        5 => color!(v, p, q),
        _ => panic!("bug in compiler"),
    }
}

const COLOR_LIST: [Color; 24] = [
    hsv_to_rgb(60.0, 100.0, 100.0),
    hsv_to_rgb(90.0, 100.0, 100.0),
    hsv_to_rgb(180.0, 100.0, 100.0),
    hsv_to_rgb(270.0, 100.0, 100.0),
    hsv_to_rgb(70.0, 100.0, 100.0),
    hsv_to_rgb(100.0, 100.0, 100.0),
    hsv_to_rgb(190.0, 100.0, 100.0),
    hsv_to_rgb(280.0, 100.0, 100.0),
    hsv_to_rgb(80.0, 100.0, 100.0),
    hsv_to_rgb(110.0, 100.0, 100.0),
    hsv_to_rgb(200.0, 100.0, 100.0),
    hsv_to_rgb(290.0, 100.0, 100.0),
    hsv_to_rgb(90.0, 100.0, 100.0),
    hsv_to_rgb(120.0, 100.0, 100.0),
    hsv_to_rgb(210.0, 100.0, 100.0),
    hsv_to_rgb(300.0, 100.0, 100.0),
    hsv_to_rgb(150.0, 100.0, 100.0),
    hsv_to_rgb(130.0, 100.0, 100.0),
    hsv_to_rgb(220.0, 100.0, 100.0),
    hsv_to_rgb(310.0, 100.0, 100.0),
    hsv_to_rgb(160.0, 100.0, 100.0),
    hsv_to_rgb(140.0, 100.0, 100.0),
    hsv_to_rgb(230.0, 100.0, 100.0),
    hsv_to_rgb(320.0, 100.0, 100.0),
];

impl Program<Message> for GardenDisplay {
    type State = ();

    fn draw(
        &self,
        _state: &Self::State,
        renderer: &Renderer,
        _theme: &Theme,
        bounds: iced::Rectangle,
        _cursor: mouse::Cursor,
    ) -> Vec<iced::widget::canvas::Geometry<Renderer>> {
        let display = self.cache.draw(renderer, bounds.size(), |frame| {
            let scale = bounds.size();
            let xscale = scale.width / self.garden.size.1 as f32;
            let yscale = scale.height / self.garden.size.0 as f32;
            let square = Size::new(xscale - 1.0, yscale - 1.0);

            for (row, line) in self.display.iter().enumerate() {
                for (col, color) in line.iter().enumerate() {
                    let point = Point::new(col as f32 * xscale, row as f32 * yscale);
                    frame.fill_rectangle(point, square, *color);
                }
            }
        });

        vec![display]
    }
}
