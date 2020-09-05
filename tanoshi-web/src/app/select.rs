use http::{Request, Response};
use web_sys::HtmlElement;
use yew::format::{Json, Nothing, Text};
use yew::prelude::*;
use yew::services::fetch::{FetchService, FetchTask};
use yew::{html, Component, ComponentLink, Html, Properties, ShouldRender};
use yew_router::components::RouterAnchor;

use tanoshi_lib::manga::SourceIndex;
use tanoshi_lib::rest::{GetSourceIndexResponse, ErrorResponse};

use super::browse::BrowseRoute;
use super::catalogue::CatalogueRoute;
use super::component::{Spinner, Toast, ToastType};

pub enum Tab {
    Installed,
    Available,
}

#[derive(Clone, Properties)]
pub struct Props {}

pub struct Select {
    fetch_task: Option<FetchTask>,
    link: ComponentLink<Self>,
    sources: Vec<SourceIndex>,
    is_fetching: bool,
    active_tab: Tab,
    button_refs: Vec<NodeRef>,
    show_toast: bool,
    toast_message: String,
}

pub enum Msg {
    SourceReady(GetSourceIndexResponse),
    ChangeToAvailableTab,
    ChangeToInstalledTab,
    InstallExtension(usize),
    ExtensionInstalled,
    ExtensionError(ErrorResponse),
    Noop,
}

impl Component for Select {
    type Message = Msg;
    type Properties = Props;

    fn create(_: Self::Properties, link: ComponentLink<Self>) -> Self {
        Select {
            fetch_task: None,
            link,
            sources: vec![],
            is_fetching: false,
            active_tab: Tab::Installed,
            button_refs: vec![NodeRef::default(), NodeRef::default()],
            show_toast: false,
            toast_message: "".to_string(),
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::SourceReady(data) => {
                self.sources = data.sources;
                self.is_fetching = false;
            }
            Msg::ChangeToInstalledTab => {
                self.active_tab = Tab::Installed;
                self.fetch_sources();
                if let Some(button) = self.button_refs[0].cast::<HtmlElement>() {
                    button
                        .class_list()
                        .add_1("bg-tachiyomi-blue-darker")
                        .expect("failed add class");
                }
                if let Some(button) = self.button_refs[1].cast::<HtmlElement>() {
                    button
                        .class_list()
                        .remove_1("bg-tachiyomi-blue-darker")
                        .expect("failed remove class");
                }
            }
            Msg::ChangeToAvailableTab => {
                self.active_tab = Tab::Available;
                self.fetch_sources();
                if let Some(button) = self.button_refs[0].cast::<HtmlElement>() {
                    button
                        .class_list()
                        .remove_1("bg-tachiyomi-blue-darker")
                        .expect("failed remove class");
                }
                if let Some(button) = self.button_refs[1].cast::<HtmlElement>() {
                    button
                        .class_list()
                        .add_1("bg-tachiyomi-blue-darker")
                        .expect("failed add class");
                }
            }
            Msg::InstallExtension(index) => {
                self.install_source(self.sources[index].name.clone());
            }
            Msg::ExtensionInstalled => {
                self.fetch_sources();
            }
            Msg::ExtensionError(res) => {
                self.show_toast = true;
                self.toast_message = res.message;
                self.is_fetching = false;
            }
            Msg::Noop => {}
        }
        true
    }

    fn change(&mut self, _props: Self::Properties) -> ShouldRender {
        false
    }

    fn view(&self) -> Html {
        html! {
            <div class="mx-auto pb-20 pt-12">
                <div class="w-full px-2 pb-2 flex justify-around block fixed inset-x-0 top-0 z-50 bg-tachiyomi-blue shadow" style="padding-top: calc(env(safe-area-inset-top) + .5rem)">
                    <button ref=self.button_refs[0].clone()
                        onclick=self.link.callback(|_| Msg::ChangeToInstalledTab)
                        class="hover:bg-tachiyomi-blue-darker rounded flex-grow bg-tachiyomi-blue-darker mr-px">
                        <span class="text-white my-2">{"Installed"}</span>
                    </button>
                    <button  ref=self.button_refs[1].clone()
                        onclick=self.link.callback(|_| Msg::ChangeToAvailableTab)
                        class="hover:bg-tachiyomi-blue-darker rounded flex-grow ml-px">
                        <span class="text-white my-2">{"Available"}</span>
                    </button>
                </div>
                <Spinner is_active=self.is_fetching is_fullscreen=true />
                {
                    match self.active_tab {
                        Tab::Installed => self.installed_view(),
                        Tab::Available => self.available_view(),
                    }
                }
                <Toast visible={self.show_toast} toast_type={ToastType::Error} message={&self.toast_message} />
            </div>
        }
    }

    fn rendered(&mut self, first_render: bool) {
        if first_render {
            self.fetch_sources();
        }
    }
}

impl Select {
    fn installed_view(&self) -> Html {
        let sources = self
            .sources
            .iter()
            .filter_map(|s| if s.installed { Some(s.clone()) } else { None })
            .collect::<Vec<SourceIndex>>();
        html! {
            <div class="flex flex-col bg-white dark:bg-gray-900 divide-y divide-gray-300 dark:divide-gray-700 border-t border-b border-gray-300 dark:border-gray-700" style="margin-top:env(safe-area-inset-top)">
            {
                for sources.iter().map(|source| html!{
                    <RouterAnchor<BrowseRoute>
                        classes="flex inline-flex justify-center p-2 content-center hover:bg-gray-200 dark:hover:bg-gray-700"
                        route=BrowseRoute::Catalogue(CatalogueRoute::Source(source.name.clone()))>
                        <div class="w-full md:w-1/2 flex justify-between items-center text-gray-900 dark:text-gray-300">
                            <div class="flex flex-col">
                                <span class="text-md font-semibold">{source.name.to_owned()}</span>
                                <span class="text-sm">{source.installed_version.to_owned()}</span>
                            </div>
                            <svg viewBox="0 0 20 20" fill="currentColor" class="chevron-right w-6 h-6"><path fill-rule="evenodd" d="M7.293 14.707a1 1 0 010-1.414L10.586 10 7.293 6.707a1 1 0 011.414-1.414l4 4a1 1 0 010 1.414l-4 4a1 1 0 01-1.414 0z" clip-rule="evenodd"></path></svg>
                        </div>
                    </RouterAnchor<BrowseRoute>>
                })
            }
            </div>
        }
    }

    fn available_view(&self) -> Html {
        html! {
            <div class="flex flex-col bg-white dark:bg-gray-900 divide-y divide-gray-300 dark:divide-gray-700 border-t border-b border-gray-300 dark:border-gray-700" style="margin-top: calc(env(safe-area-inset-top) + .5rem)">
            {
                for (0..self.sources.len()).map(|i| html!{
                    <div class="flex inline-flex justify-center p-2 content-center hover:bg-gray-200 dark:hover:bg-gray-700">
                    <div class="w-full md:w-1/2 flex justify-between items-center text-gray-900 dark:text-gray-300">
                        <div class="flex flex-col">
                            <span class="text-md font-semibold text-gray-900 dark:text-gray-300">{self.sources[i].name.clone()}</span>
                            <span class="text-sm">{self.sources[i].version.clone()}</span>
                        </div>
                        <button class="bg-tachiyomi-blue text-white p-2 rounded-full"
                            disabled={!self.sources[i].update && self.sources[i].installed}
                            onclick={self.link.callback(move |_| Msg::InstallExtension(i))}>
                            {
                                if self.sources[i].update {
                                    "Update"
                                } else if !self.sources[i].update && !self.sources[i].installed{
                                    "Install"
                                } else if !self.sources[i].update && self.sources[i].installed {
                                    "Installed"
                                } else {
                                    ""
                                }
                            }
                        </button>
                    </div>
                    </div>
                })
            }
            </div>
        }
    }

    fn fetch_sources(&mut self) {
        let req = Request::get("/api/source")
            .body(Nothing)
            .expect("failed to build request");

        if let Ok(task) = FetchService::fetch(
            req,
            self.link.callback(
                |response: Response<Json<Result<GetSourceIndexResponse, anyhow::Error>>>| {
                    if let (meta, Json(Ok(data))) = response.into_parts() {
                        if meta.status.is_success() {
                            return Msg::SourceReady(data);
                        }
                    }
                    Msg::Noop
                },
            ),
        ) {
            self.fetch_task = Some(FetchTask::from(task));
            self.is_fetching = true;
        }
    }

    fn install_source(&mut self, name: String) {
        let req = Request::post(format!("/api/source/install/{}", &name))
            .body(Nothing)
            .expect("failed to build request");

        if let Ok(task) = FetchService::fetch(
            req,
            self.link.callback(|response: Response<Text>| {
                if let (meta, body) = response.into_parts() {
                    if meta.status.is_success() {
                        return Msg::ExtensionInstalled;
                    } else {
                        let res: ErrorResponse = serde_json::from_str(&body.unwrap()).unwrap();
                        return Msg::ExtensionError(res);
                    }
                }
                Msg::Noop
            }),
        ) {
            self.fetch_task = Some(FetchTask::from(task));
            self.is_fetching = true;
        }
    }
}
