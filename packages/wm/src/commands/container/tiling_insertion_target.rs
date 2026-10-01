use anyhow::Context;
use wm_common::{TilingDirection, TilingLayout};

use super::set_tiling_direction;
use crate::{
  models::{Container, TilingWindow},
  traits::{CommonGetters, PositionGetters},
  user_config::UserConfig,
  wm_state::WmState,
};

/// Gets where to insert a tiling window that is automatically placed
/// beside `sibling` (e.g. a newly opened window).
///
/// With `TilingLayout::Dwindle`, the tiling direction around `sibling` is
/// first changed to run along its longer side. This can wrap `sibling` in
/// a new split container, or change its workspace's tiling direction if
/// it's the workspace's only tiling child.
///
/// Returns tuple of (parent container, insertion index), where the index
/// is directly after `sibling`.
pub fn tiling_insertion_target(
  sibling: &TilingWindow,
  state: &mut WmState,
  config: &UserConfig,
) -> anyhow::Result<(Container, usize)> {
  if config.value.window_behavior.tiling_layout == TilingLayout::Dwindle {
    let tiling_direction =
      TilingDirection::from_longer_side(&sibling.to_rect()?);

    set_tiling_direction(
      sibling.clone().into(),
      state,
      config,
      &tiling_direction,
    )?;
  }

  Ok((sibling.parent().context("No parent.")?, sibling.index() + 1))
}

#[cfg(test)]
mod tests {
  use tokio::sync::mpsc;
  use wm_common::{
    ParsedConfig, TilingDirection, TilingLayout, WindowBehaviorConfig,
  };
  use wm_platform::{Dispatcher, Rect};

  use super::tiling_insertion_target;
  use crate::{
    models::{Container, Monitor, TilingWindow, Workspace},
    traits::{CommonGetters, TilingDirectionGetters, TilingSizeGetters},
    user_config::UserConfig,
    wm_state::WmState,
  };

  /// Creates a `WmState` for commands that only need it to emit events.
  ///
  /// The container tree is built separately via mocks, so the state's own
  /// root stays empty.
  fn mock_state() -> WmState {
    let (event_tx, _) = mpsc::unbounded_channel();
    let (exit_tx, _) = mpsc::unbounded_channel();
    WmState::new(Dispatcher::mock(), event_tx, exit_tx)
  }

  /// Creates a `UserConfig` with the given tiling layout.
  fn mock_config(tiling_layout: TilingLayout) -> UserConfig {
    UserConfig::mock()
      .value(ParsedConfig {
        window_behavior: WindowBehaviorConfig {
          tiling_layout,
          ..WindowBehaviorConfig::default()
        },
        ..ParsedConfig::default()
      })
      .call()
  }

  /// Creates a workspace with the given windows on a monitor with the
  /// given working area.
  fn mock_workspace(
    working_area: Rect,
    tiling_direction: TilingDirection,
    windows: &[TilingWindow],
  ) -> Workspace {
    let workspace = Workspace::mock()
      .tiling_direction(tiling_direction)
      .tiling_containers(windows.iter().cloned().map(Into::into).collect())
      .call();

    Monitor::mock()
      .bounds(working_area.clone())
      .working_area(working_area)
      .workspaces(vec![workspace.clone()])
      .call();

    workspace
  }

  fn landscape() -> Rect {
    Rect::from_xy(0, 0, 1600, 1000)
  }

  fn portrait() -> Rect {
    Rect::from_xy(0, 0, 1000, 1600)
  }

  #[test]
  fn manual_layout_inserts_after_sibling_without_changes() {
    let first = TilingWindow::mock().call();
    let second = TilingWindow::mock().call();
    let workspace = mock_workspace(
      landscape(),
      TilingDirection::Horizontal,
      &[first.clone(), second],
    );

    let (parent, index) = tiling_insertion_target(
      &first,
      &mut mock_state(),
      &mock_config(TilingLayout::Manual),
    )
    .unwrap();

    assert_eq!(parent, Container::from(workspace.clone()));
    assert_eq!(index, 1);
    assert_eq!(workspace.child_count(), 2);
    assert_eq!(workspace.tiling_direction(), TilingDirection::Horizontal);
  }

  #[test]
  fn dwindle_keeps_direction_matching_longer_side() {
    // Only window on a landscape workspace is wider than it is tall.
    let window = TilingWindow::mock().call();
    let workspace = mock_workspace(
      landscape(),
      TilingDirection::Horizontal,
      std::slice::from_ref(&window),
    );

    let (parent, index) = tiling_insertion_target(
      &window,
      &mut mock_state(),
      &mock_config(TilingLayout::Dwindle),
    )
    .unwrap();

    assert_eq!(parent, Container::from(workspace.clone()));
    assert_eq!(index, 1);
    assert_eq!(workspace.tiling_direction(), TilingDirection::Horizontal);
  }

  #[test]
  fn dwindle_wraps_tall_sibling_in_vertical_split() {
    // H[first, second] on a landscape workspace: each half is taller than
    // it is wide, so `first` gets wrapped as H[V[first], second].
    let first = TilingWindow::mock().call();
    let second = TilingWindow::mock().call();
    let workspace = mock_workspace(
      landscape(),
      TilingDirection::Horizontal,
      &[first.clone(), second.clone()],
    );
    let first_size = first.tiling_size();

    let (parent, index) = tiling_insertion_target(
      &first,
      &mut mock_state(),
      &mock_config(TilingLayout::Dwindle),
    )
    .unwrap();

    let split = parent.as_split().expect("Parent should be a split.");
    assert_eq!(split.tiling_direction(), TilingDirection::Vertical);
    assert_eq!(index, 1);
    assert_eq!(split.children(), vec![Container::from(first)]);
    assert!((split.tiling_size() - first_size).abs() < f32::EPSILON);

    // Workspace is unchanged apart from `first` being wrapped.
    assert_eq!(workspace.tiling_direction(), TilingDirection::Horizontal);
    assert_eq!(
      workspace.children(),
      vec![Container::from(split.clone()), Container::from(second)]
    );
  }

  #[test]
  fn dwindle_flips_workspace_for_only_child() {
    // Only window on a portrait workspace is taller than it is wide, so
    // the workspace itself becomes vertical instead of gaining a split.
    let window = TilingWindow::mock().call();
    let workspace = mock_workspace(
      portrait(),
      TilingDirection::Horizontal,
      std::slice::from_ref(&window),
    );

    let (parent, index) = tiling_insertion_target(
      &window,
      &mut mock_state(),
      &mock_config(TilingLayout::Dwindle),
    )
    .unwrap();

    assert_eq!(parent, Container::from(workspace.clone()));
    assert_eq!(index, 1);
    assert_eq!(workspace.tiling_direction(), TilingDirection::Vertical);
    assert_eq!(workspace.children(), vec![Container::from(window)]);
  }
}
