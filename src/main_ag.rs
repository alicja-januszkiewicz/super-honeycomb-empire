#[macroquad::main(window_conf)]
async fn main() {
    let (mut exit, ui) = main_menu(&mut assets).await;

    if exit {return};

    let mut app = App::from_ui(ui, &mut assets);

    let mut layout = assets.init_layout.clone();

    let mut time = 0.0;
    while !exit {
        app.draw(&mut layout, &mut assets, time);
        app.update();
        if is_key_pressed(KeyCode::F1) {
            app = app.swap();
        }
        time += get_frame_time();
        exit = backend.poll_events(|event| {
            game.poll(event, &mut layout);
        });

        app.update();
        app.draw(&mut backend, &mut layout, time);
        backend.next_frame().await;
    }

}