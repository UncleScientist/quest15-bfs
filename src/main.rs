use std::collections::{HashSet, VecDeque};

use iced::{
    color, mouse,
    widget::{
        canvas::{Cache, Program},
        Canvas,
    },
    Color, Element, Length, Point, Renderer, Size, Subscription, Task, Theme,
};

use crate::garden::Garden;

mod garden;

fn main() {
    let _ = iced::application("BFS Viz", GardenDisplay::update, GardenDisplay::view)
        .subscription(GardenDisplay::subscription)
        .run();
}

#[derive(Debug)]
enum SearchResult {
    Found(usize),
    Continue,
    Failed,
}

#[derive(Debug)]
enum Message {
    Tick,
}

#[derive(Debug)]
struct GardenDisplay {
    cache: Cache,
    garden: Garden,
    running: bool,
    iterations: usize,
    queue: VecDeque<(usize, (u64, (i64, i64)))>,
    state_visited: HashSet<(u64, (i64, i64))>,
    loc_visited: HashSet<(i64, i64)>,
}

impl Default for GardenDisplay {
    fn default() -> Self {
        let data = std::fs::read_to_string("input/everybody_codes_e2024_q15_p2.txt").expect("file");
        let garden = Garden::parse(&data);

        let queue = VecDeque::from([(0usize, (garden.herb_types, garden.start))]);

        Self {
            cache: Cache::default(),
            garden,
            running: true,
            iterations: 0,
            queue,
            state_visited: HashSet::new(),
            loc_visited: HashSet::new(),
        }
    }
}

impl GardenDisplay {
    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Tick => {
                if self.running {
                    self.cache.clear();
                    for _ in 0..5 {
                        self.iterations += 1;
                        match self.step() {
                            SearchResult::Found(steps) => {
                                println!(
                                    "found in {steps} steps, after {} iterations",
                                    self.iterations
                                );
                                self.running = false;
                            }
                            SearchResult::Continue => {}
                            SearchResult::Failed => {
                                self.running = false;
                            }
                        }
                    }
                }
            }
        }
        Task::none()
    }

    fn view(&self) -> Element<'_, Message> {
        Canvas::new(self)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }

    fn subscription(&self) -> Subscription<Message> {
        iced::time::every(std::time::Duration::from_millis(25)).map(|_| Message::Tick)
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
            SearchResult::Failed
        }
    }
}

const COLOR_LIST: [Color; 8] = [
    Color {
        r: 0.7,
        g: 0.4,
        b: 1.0,
        a: 1.0,
    },
    Color {
        r: 0.6,
        g: 0.5,
        b: 0.9,
        a: 1.0,
    },
    Color {
        r: 0.5,
        g: 0.6,
        b: 0.8,
        a: 1.0,
    },
    Color {
        r: 0.4,
        g: 0.7,
        b: 0.7,
        a: 1.0,
    },
    Color {
        r: 0.3,
        g: 0.8,
        b: 0.6,
        a: 1.0,
    },
    Color {
        r: 0.2,
        g: 0.9,
        b: 0.5,
        a: 1.0,
    },
    Color {
        r: 0.1,
        g: 1.0,
        b: 0.4,
        a: 1.0,
    },
    Color {
        r: 0.0,
        g: 0.9,
        b: 0.3,
        a: 1.0,
    },
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
            for loc in &self.garden.maze {
                let point = Point::new(loc.1 as f32 * xscale, loc.0 as f32 * yscale);
                frame.fill_rectangle(point, square, color!(0x005000));
            }

            for loc in &self.loc_visited {
                let point = Point::new(loc.1 as f32 * xscale, loc.0 as f32 * yscale);
                frame.fill_rectangle(point, square, color!(0x008000));
            }

            for (loc, value) in &self.garden.herbs {
                let point = Point::new(loc.1 as f32 * xscale, loc.0 as f32 * yscale);
                let idx = (*value as u8 - b'A') as usize;
                frame.fill_rectangle(point, square, COLOR_LIST[idx]);
            }

            for (_, (_, loc)) in &self.queue {
                let point = Point::new(loc.1 as f32 * xscale, loc.0 as f32 * yscale);
                frame.fill_rectangle(point, square, color!(0x0000a0));
            }
        });

        vec![display]
    }
}
