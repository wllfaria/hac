use anathema::component::Component;

pub struct Testing;

impl Component for Testing {
    type Message = ();
    type State = ();

    fn on_focus(
        &mut self,
        _: &mut Self::State,
        _: anathema::component::Children<'_, '_>,
        _: anathema::component::Context<'_, '_, Self::State>,
    ) {
    }
}
