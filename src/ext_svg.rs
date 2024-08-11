use iced::{
    advanced::{svg, Widget},
    widget::Svg,
    Element,
};

pub(crate) struct ExtendedSvg<Theme = iced_style::Theme>
where
    Theme: iced_style::svg::StyleSheet,
{
    pub inner: Svg<Theme>,
}

impl<Message, Theme, Renderer> Widget<Message, Theme, Renderer> for ExtendedSvg<Theme>
where
    Theme: iced_style::svg::StyleSheet,
    Renderer: svg::Renderer,
{
    fn size(&self) -> iced::Size<iced::Length> {
        <Svg<Theme> as Widget<Message, Theme, Renderer>>::size(&self.inner)
    }

    fn layout(
        &self,
        tree: &mut iced::advanced::widget::Tree,
        renderer: &Renderer,
        limits: &iced::advanced::layout::Limits,
    ) -> iced::advanced::layout::Node {
        <Svg<Theme> as Widget<Message, Theme, Renderer>>::layout(
            &self.inner,
            tree,
            renderer,
            limits,
        )
    }

    fn draw(
        &self,
        tree: &iced::advanced::widget::Tree,
        renderer: &mut Renderer,
        theme: &Theme,
        style: &iced::advanced::renderer::Style,
        layout: iced::advanced::Layout<'_>,
        cursor: iced::advanced::mouse::Cursor,
        viewport: &iced::Rectangle,
    ) {
        <Svg<Theme> as Widget<Message, Theme, Renderer>>::draw(
            &self.inner,
            tree,
            renderer,
            theme,
            style,
            layout,
            cursor,
            viewport,
        )
    }

    fn on_event(
        &mut self,
        _state: &mut iced::advanced::widget::Tree,
        _event: iced::Event,
        layout: iced::advanced::Layout<'_>,
        cursor: iced::advanced::mouse::Cursor,
        _renderer: &Renderer,
        _clipboard: &mut dyn iced::advanced::Clipboard,
        _shell: &mut iced::advanced::Shell<'_, Message>,
        _viewport: &iced::Rectangle,
    ) -> iced_style::core::event::Status {
        let position = cursor.position();
        if position.is_some() && layout.bounds().contains(position.unwrap()) {
            iced::event::Status::Captured
        } else {
            iced::event::Status::Ignored
        }
    }
}

impl<'a, Message, Theme, Renderer> From<ExtendedSvg<Theme>>
    for Element<'a, Message, Theme, Renderer>
where
    Theme: iced_style::svg::StyleSheet + 'a,
    Renderer: svg::Renderer + 'a,
{
    fn from(icon: ExtendedSvg<Theme>) -> Element<'a, Message, Theme, Renderer> {
        Element::new(icon)
    }
}
