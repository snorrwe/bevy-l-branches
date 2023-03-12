use bevy::prelude::*;
use game::LAUNCHER_TITLE;
use stylist::yew::styled_component;
use stylist::{css, global_style};
use yew::prelude::*;

use game::transport::{Event, EventHandle};

fn set_window_title(title: &str) {
    web_sys::window()
        .map(|w| w.document())
        .flatten()
        .expect("Unable to get DOM")
        .set_title(title);
}

fn set_global_css() {
    global_style! {
        r#"
        html {
            min-height: 100%;
            position: relative;
        }
        body {
            height: 100%;
            padding: 0;
            margin: 0;
        }
        "#
    }
    .expect("Unable to mount global style");
}

#[derive(Clone, Properties)]
struct RootProps {
    ev: EventHandle,
}

impl PartialEq for RootProps {
    fn eq(&self, other: &Self) -> bool {
        (&self.ev.sender as *const _) == (&other.ev.sender as *const _)
    }
}

#[styled_component(Root)]
fn view(props: &RootProps) -> Html {
    set_window_title(LAUNCHER_TITLE);
    set_global_css();

    let css = css!(
        r#"
        position: absolute;
        overflow: hidden;
        width: 100%;
        height: 100%;
        "#
    );

    let handler = props.ev.clone();

    html! {
        <div class={ css }>
            <button onclick={Callback::from(move |_|{
                handler.sender.try_send(Event::Hello).unwrap();
            })}>{"Hello"}</button>
            <canvas id="bevy"></canvas>
        </div>
    }
}

fn main() {
    let handle = EventHandle::new(16);
    let props = RootProps { ev: handle.clone() };
    // Mount the DOM
    let app = yew::Renderer::<Root>::with_props(props);
    app.render().send_future(async move {
        // Start the Bevy App
        let mut app = game::app(false, handle);
        info!("Starting launcher: WASM");
        app.run();
    });
}
