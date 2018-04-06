/*
 * Copyright (c) 2017-2018 Boucher, Antoni <bouanto@zoho.com>
 *
 * Permission is hereby granted, free of charge, to any person obtaining a copy of
 * this software and associated documentation files (the "Software"), to deal in
 * the Software without restriction, including without limitation the rights to
 * use, copy, modify, merge, publish, distribute, sublicense, and/or sell copies of
 * the Software, and to permit persons to whom the Software is furnished to do so,
 * subject to the following conditions:
 *
 * The above copyright notice and this permission notice shall be included in all
 * copies or substantial portions of the Software.
 *
 * THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
 * IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY, FITNESS
 * FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR
 * COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER
 * IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN
 * CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.
 */

#![feature(proc_macro)]

extern crate gtk;
#[macro_use]
extern crate relm;
extern crate relm_attributes;
#[macro_use]
extern crate relm_derive;
#[macro_use]
extern crate relm_test;

use gtk::{
    ButtonExt,
    Inhibit,
    LabelExt,
    OrientableExt,
    WidgetExt,
};
use gtk::Orientation::Vertical;
use relm::{Relm, Widget, timeout};
use relm_attributes::widget;

use self::Msg::*;

#[derive(Clone)]
pub struct Model {
    counter: i32,
    relm: Relm<Win>,
    text: String,
}

#[derive(Clone, Msg)]
pub enum Msg {
    Decrement,
    FiveInc,
    GetModel,
    Increment,
    RecvModel(Model),
    Quit,
    TwoInc(i32, i32),
    UpdateText,
    UpdateTextNow,
}

#[widget]
impl Widget for Win {
    fn model(relm: &Relm<Self>, _: ()) -> Model {
        Model {
            counter: 0,
            relm: relm.clone(),
            text: String::new(),
        }
    }

    fn update(&mut self, event: Msg) {
        match event {
            Decrement => self.model.counter -= 1,
            // To be listened to by the user.
            FiveInc => (),
            GetModel => self.model.relm.stream().emit(RecvModel(self.model.clone())),
            Increment => {
                self.model.counter += 1;
                if self.model.counter == 2 {
                    self.model.relm.stream().emit(TwoInc(1, 2));
                }
                if self.model.counter == 5 {
                    self.model.relm.stream().emit(FiveInc);
                }
            },
            // To be listened to by the user.
            RecvModel(_) => (),
            Quit => gtk::main_quit(),
            // To be listened to by the user.
            TwoInc(_, _) => (),
            UpdateText => timeout(self.model.relm.stream(), 100, || UpdateTextNow),
            UpdateTextNow => self.model.text = "Updated text".to_string(),
        }
    }

    view! {
        gtk::Window {
            gtk::Box {
                orientation: Vertical,
                #[name="inc_button"]
                gtk::Button {
                    clicked => Increment,
                    label: "+",
                },
                #[name="label"]
                gtk::Label {
                    text: &self.model.counter.to_string(),
                },
                #[name="dec_button"]
                gtk::Button {
                    clicked => Decrement,
                    label: "-",
                },
                #[name="text"]
                gtk::Label {
                    text: &self.model.text,
                },
                #[name="update_button"]
                gtk::Button {
                    clicked => UpdateText,
                    label: "Update text",
                },
            },
            delete_event(_, _) => (Quit, Inhibit(false)),
        }
    }
}

#[cfg(test)]
mod tests {
    use gtk::LabelExt;

    use relm;
    use relm_test::{Observer, click, wait};

    use Msg::{self, FiveInc, GetModel, RecvModel, TwoInc};
    use Win;

    #[test]
    fn label_change() {
        let (component, widgets) = relm::init_test::<Win>(()).unwrap();
        let inc_button = &widgets.inc_button;
        let dec_button = &widgets.dec_button;
        let update_button = &widgets.update_button;

        // Observe for messages.
        let observer = Observer::new(component.stream(), |msg|
            if let FiveInc = msg {
                true
            }
            else {
                false
            }
        );

        // Shortcut for the previous call to Observer::new().
        let two_observer = observer_new!(component, TwoInc(_, _));

        let model_observer = Observer::new(component.stream(), |msg|
            if let RecvModel(_) = msg {
                true
            }
            else {
                false
            }
        );

        assert_text!(widgets.label, 0);
        click(inc_button);
        assert_text!(widgets.label, 1);
        click(inc_button);
        assert_text!(widgets.label, 2);

        // Shortcut for the call to wait() below.
        observer_wait!(let TwoInc(one, two) = two_observer);
        assert_eq!(one, 1);
        assert_eq!(two, 2);

        click(dec_button);
        assert_text!(widgets.label, 1);
        click(inc_button);
        assert_text!(widgets.label, 2);

        observer_wait!(let Msg::TwoInc(one, two) = two_observer);
        assert_eq!(one, 1);
        assert_eq!(two, 2);

        click(dec_button);
        assert_text!(widgets.label, 1);
        click(dec_button);
        assert_text!(widgets.label, 0);
        click(dec_button);
        assert_text!(widgets.label, -1);

        for _ in 0..6 {
            click(inc_button);
        }

        // Wait to receive the message on this observer.
        observer.wait();

        // Ask for the model. This will emit RecvModel.
        component.stream().emit(GetModel);

        let msg = model_observer.wait();
        if let RecvModel(model) = msg {
            assert_eq!(model.counter, 5);
        }
        else {
            panic!("Wrong message type.");
        }

        component.stream().emit(GetModel);
        observer_wait!(let RecvModel(model) = model_observer);
        assert_eq!(model.counter, 5);

        assert_text!(widgets.text, "");
        click(update_button);
        assert_text!(widgets.text, "");

        wait(200);
        assert_text!(widgets.text, "Updated text");
    }
}
