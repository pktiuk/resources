use std::time::{Duration, SystemTime};

use adw::{prelude::*, subclass::prelude::*};
use gtk::glib;

use crate::config::PROFILE;
use crate::i18n::{i18n, i18n_f};
use crate::utils::network::{NetworkData, NetworkInterface};
use crate::utils::units::{convert_speed, convert_storage};
use crate::utils::NaNDefault;

mod imp {
    use std::cell::{Cell, RefCell};

    use crate::ui::widgets::graph_box::ResGraphBox;

    use super::*;

    use gtk::{
        gio::{Icon, ThemedIcon},
        glib::{ParamSpec, Properties, Value},
        CompositeTemplate,
    };

    #[derive(CompositeTemplate, Properties)]
    #[template(resource = "/net/nokyan/Resources/ui/pages/network.ui")]
    #[properties(wrapper_type = super::ResNetwork)]
    pub struct ResNetwork {
        #[template_child]
        pub receiving: TemplateChild<ResGraphBox>,
        #[template_child]
        pub sending: TemplateChild<ResGraphBox>,
        #[template_child]
        pub total_received: TemplateChild<adw::ActionRow>,
        #[template_child]
        pub total_sent: TemplateChild<adw::ActionRow>,
        #[template_child]
        pub manufacturer: TemplateChild<adw::ActionRow>,
        #[template_child]
        pub driver: TemplateChild<adw::ActionRow>,
        #[template_child]
        pub interface: TemplateChild<adw::ActionRow>,
        #[template_child]
        pub hw_address: TemplateChild<adw::ActionRow>,
        pub old_received_bytes: Cell<usize>,
        pub old_sent_bytes: Cell<usize>,
        pub last_timestamp: Cell<SystemTime>,
        pub network_interface: RefCell<NetworkInterface>,

        #[property(get)]
        uses_progress_bar: Cell<bool>,

        #[property(get)]
        main_graph_color: glib::Bytes,

        #[property(get = Self::icon, set = Self::set_icon, type = Icon)]
        icon: RefCell<Icon>,

        #[property(get, set)]
        usage: Cell<f64>,

        #[property(get = Self::tab_name, set = Self::set_tab_name, type = glib::GString)]
        tab_name: Cell<glib::GString>,

        #[property(get = Self::tab_detail, set = Self::set_tab_detail, type = glib::GString)]
        tab_detail_string: Cell<glib::GString>,

        #[property(get = Self::tab_usage_string, set = Self::set_tab_usage_string, type = glib::GString)]
        tab_usage_string: Cell<glib::GString>,
    }

    impl ResNetwork {
        pub fn tab_name(&self) -> glib::GString {
            let tab_name = self.tab_name.take();
            let result = tab_name.clone();
            self.tab_name.set(tab_name);
            result
        }

        pub fn set_tab_name(&self, tab_name: &str) {
            self.tab_name.set(glib::GString::from(tab_name));
        }

        pub fn tab_detail(&self) -> glib::GString {
            let detail = self.tab_detail_string.take();
            let result = detail.clone();
            self.tab_detail_string.set(detail);
            result
        }

        pub fn set_tab_detail(&self, detail: &str) {
            self.tab_detail_string.set(glib::GString::from(detail));
        }

        pub fn icon(&self) -> Icon {
            let icon = self.icon.replace_with(|_| NetworkInterface::default_icon());
            let result = icon.clone();
            self.icon.set(icon);
            result
        }

        pub fn set_icon(&self, icon: &Icon) {
            self.icon.set(icon.clone());
        }

        pub fn tab_usage_string(&self) -> glib::GString {
            let tab_usage_string = self.tab_usage_string.take();
            let result = tab_usage_string.clone();
            self.tab_usage_string.set(tab_usage_string);
            result
        }

        pub fn set_tab_usage_string(&self, tab_usage_string: &str) {
            self.tab_usage_string
                .set(glib::GString::from(tab_usage_string));
        }
    }

    impl Default for ResNetwork {
        fn default() -> Self {
            Self {
                receiving: Default::default(),
                sending: Default::default(),
                total_received: Default::default(),
                total_sent: Default::default(),
                manufacturer: Default::default(),
                driver: Default::default(),
                interface: Default::default(),
                hw_address: Default::default(),
                uses_progress_bar: Cell::new(true),
                main_graph_color: glib::Bytes::from_static(&super::ResNetwork::MAIN_GRAPH_COLOR),
                icon: RefCell::new(ThemedIcon::new("unknown-network-type-symbolic").into()),
                usage: Default::default(),
                tab_name: Cell::new(glib::GString::from(i18n("Network Interface"))),
                tab_detail_string: Cell::new(glib::GString::from("")),
                old_received_bytes: Cell::default(),
                old_sent_bytes: Cell::default(),
                last_timestamp: Cell::new(
                    SystemTime::now()
                        .checked_sub(Duration::from_secs(1))
                        .unwrap(),
                ),
                network_interface: RefCell::default(),
                tab_usage_string: Cell::new(glib::GString::from("")),
            }
        }
    }

    #[glib::object_subclass]
    impl ObjectSubclass for ResNetwork {
        const NAME: &'static str = "ResNetwork";
        type Type = super::ResNetwork;
        type ParentType = adw::Bin;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
        }

        // You must call `Widget`'s `init_template()` within `instance_init()`.
        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for ResNetwork {
        fn constructed(&self) {
            self.parent_constructed();
            let obj = self.obj();

            // Devel Profile
            if PROFILE == "Devel" {
                obj.add_css_class("devel");
            }
        }

        fn properties() -> &'static [ParamSpec] {
            Self::derived_properties()
        }

        fn set_property(&self, id: usize, value: &Value, pspec: &ParamSpec) {
            self.derived_set_property(id, value, pspec);
        }

        fn property(&self, id: usize, pspec: &ParamSpec) -> Value {
            self.derived_property(id, pspec)
        }
    }

    impl WidgetImpl for ResNetwork {}
    impl BinImpl for ResNetwork {}
}

glib::wrapper! {
    pub struct ResNetwork(ObjectSubclass<imp::ResNetwork>)
        @extends gtk::Widget, adw::Bin;
}

impl ResNetwork {
    // TODO: this is the color for receiving, but it is also used in sidebar,
    // which graphs the sum of send+recv.
    // This does not make much sense, but we probably can't do something
    // like separate send/receive lines without some refactoring to ResGraph.
    const MAIN_GRAPH_COLOR: [u8; 3] = [52, 170, 175];

    pub fn new() -> Self {
        glib::Object::new::<Self>()
    }

    pub fn init(&self, network_data: &NetworkData) {
        self.setup_widgets(network_data);
    }

    pub fn setup_widgets(&self, network_data: &NetworkData) {
        let imp = self.imp();
        let network_interface = &network_data.inner;

        self.imp().set_icon(&network_interface.icon());

        imp.set_tab_name(&i18n(&network_interface.interface_type.to_string()));

        imp.receiving.set_title_label(&i18n("Receiving"));
        imp.receiving.graph().set_graph_color(
            Self::MAIN_GRAPH_COLOR[0],
            Self::MAIN_GRAPH_COLOR[1],
            Self::MAIN_GRAPH_COLOR[2],
        );
        imp.receiving.graph().set_locked_max_y(None);

        imp.sending.set_title_label(&i18n("Sending"));
        imp.sending.graph().set_graph_color(222, 77, 119);
        imp.sending.graph().set_locked_max_y(None);

        imp.manufacturer.set_subtitle(
            &network_interface
                .vendor
                .as_ref()
                .cloned()
                .unwrap_or_else(|| i18n("N/A")),
        );

        imp.driver.set_subtitle(
            &network_interface
                .driver_name
                .as_ref()
                .cloned()
                .unwrap_or_else(|| i18n("N/A")),
        );

        imp.interface.set_subtitle(
            network_interface
                .interface_name
                .to_str()
                .unwrap_or(&i18n("N/A")),
        );

        let hw_address = network_interface
            .hw_address
            .as_ref()
            .cloned()
            .unwrap_or_else(|| i18n("N/A"));

        if hw_address.is_empty() {
            imp.hw_address.set_subtitle(&i18n("N/A"));
        } else {
            imp.hw_address.set_subtitle(&hw_address);
        }

        imp.last_timestamp.set(
            SystemTime::now()
                .checked_sub(Duration::from_secs(1))
                .unwrap(),
        );

        imp.old_received_bytes.set(network_data.received_bytes);
        imp.old_sent_bytes.set(network_data.sent_bytes);

        imp.set_tab_detail(&network_data.display_name);
    }

    pub fn refresh_page(&self, network_data: NetworkData) {
        let NetworkData {
            received_bytes,
            sent_bytes,

            inner: _,
            is_virtual: _,
            display_name: _,
        } = network_data;

        let imp = self.imp();
        let time_passed = SystemTime::now()
            .duration_since(imp.last_timestamp.get())
            .map_or(1.0f64, |timestamp| timestamp.as_secs_f64());

        let received_delta =
            (received_bytes.saturating_sub(imp.old_received_bytes.get())) as f64 / time_passed;
        let sent_delta = (sent_bytes.saturating_sub(imp.old_sent_bytes.get())) as f64 / time_passed;

        imp.total_received
            .set_subtitle(&convert_storage(received_bytes as f64, false));
        imp.total_sent
            .set_subtitle(&convert_storage(sent_bytes as f64, false));

        imp.receiving.graph().push_data_point(received_delta);
        let highest_received = imp.receiving.graph().get_highest_value();
        imp.receiving.set_subtitle(&format!(
            "{} · {} {}",
            convert_speed(received_delta, true),
            i18n("Highest:"),
            convert_speed(highest_received, true)
        ));

        imp.sending.graph().push_data_point(sent_delta);
        let highest_sent = imp.sending.graph().get_highest_value();
        imp.sending.set_subtitle(&format!(
            "{} · {} {}",
            convert_speed(sent_delta, true),
            i18n("Highest:"),
            convert_speed(highest_sent, true)
        ));

        self.set_property(
            "usage",
            f64::max(received_delta / highest_received, sent_delta / highest_sent).nan_default(0.0),
        );

        self.set_property(
            "tab_usage_string",
            i18n_f(
                // Translators: This is an abbreviation for "Receive" and "Send". This is displayed in the sidebar so
                // your translation should preferably be quite short or an abbreviation
                "R: {} · S: {}",
                &[
                    &convert_speed(received_delta, true),
                    &convert_speed(sent_delta, true),
                ],
            ),
        );

        imp.old_received_bytes.set(received_bytes);
        imp.old_sent_bytes.set(sent_bytes);
        imp.last_timestamp.set(SystemTime::now());
    }
}
