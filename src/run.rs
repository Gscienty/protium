use crate::{ColorSpace, Game, atlas, game, ui};

pub fn run_game<G>(game: G)
where
    G: Game + 'static,
{
    let eventloop = ui::EventLoop::new(game::ImplUIGame::new(game));

    let _ = eventloop.run();
}

pub fn init(canvas: &web_sys::HtmlCanvasElement, space: ColorSpace, max_vertex_count_limit: usize) {
    ui::init_input_state(canvas);
    atlas::init_material(canvas, space, max_vertex_count_limit);
}
