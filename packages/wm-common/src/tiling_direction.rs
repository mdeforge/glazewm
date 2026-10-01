use std::str::FromStr;

use anyhow::bail;
use serde::{Deserialize, Serialize};
use wm_platform::{Direction, Rect};

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TilingDirection {
  Horizontal,
  Vertical,
}

impl TilingDirection {
  /// Gets the inverse of a given tiling direction.
  ///
  /// Example:
  /// ```
  /// # use wm_common::TilingDirection;
  /// let dir = TilingDirection::Horizontal.inverse();
  /// assert_eq!(dir, TilingDirection::Vertical);
  /// ```
  #[must_use]
  pub fn inverse(&self) -> Self {
    match self {
      Self::Horizontal => Self::Vertical,
      Self::Vertical => Self::Horizontal,
    }
  }

  /// Gets the tiling direction that is needed when moving or shifting
  /// focus in a given direction.
  ///
  /// Example:
  /// ```
  /// # use wm_common::TilingDirection;
  /// # use wm_platform::Direction;
  /// let dir = TilingDirection::from_direction(&Direction::Left);
  /// assert_eq!(dir, TilingDirection::Horizontal);
  /// ```
  #[must_use]
  pub fn from_direction(direction: &Direction) -> Self {
    match direction {
      Direction::Left | Direction::Right => Self::Horizontal,
      Direction::Up | Direction::Down => Self::Vertical,
    }
  }

  /// Gets the tiling direction that runs along the longer side of a
  /// rect. Splitting the rect in this direction gives the two halves that
  /// are closest to square.
  ///
  /// Square rects get `TilingDirection::Vertical`.
  ///
  /// Example:
  /// ```
  /// # use wm_common::TilingDirection;
  /// # use wm_platform::Rect;
  /// let wide = Rect::from_xy(0, 0, 1920, 1080);
  /// let dir = TilingDirection::from_longer_side(&wide);
  /// assert_eq!(dir, TilingDirection::Horizontal);
  ///
  /// let tall = Rect::from_xy(0, 0, 1080, 1920);
  /// let dir = TilingDirection::from_longer_side(&tall);
  /// assert_eq!(dir, TilingDirection::Vertical);
  /// ```
  #[must_use]
  pub fn from_longer_side(rect: &Rect) -> Self {
    if rect.width() > rect.height() {
      Self::Horizontal
    } else {
      Self::Vertical
    }
  }
}

impl FromStr for TilingDirection {
  type Err = anyhow::Error;

  /// Parses a string into a tiling direction.
  ///
  /// Example:
  /// ```
  /// # use wm_common::TilingDirection;
  /// # use std::str::FromStr;
  /// let dir = TilingDirection::from_str("horizontal");
  /// assert_eq!(dir.unwrap(), TilingDirection::Horizontal);
  ///
  /// let dir = TilingDirection::from_str("vertical");
  /// assert_eq!(dir.unwrap(), TilingDirection::Vertical);
  /// ```
  fn from_str(unparsed: &str) -> anyhow::Result<Self> {
    match unparsed {
      "horizontal" => Ok(Self::Horizontal),
      "vertical" => Ok(Self::Vertical),
      _ => bail!("Not a valid tiling direction: {}", unparsed),
    }
  }
}
