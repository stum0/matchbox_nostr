#![warn(missing_docs)]
#![doc = include_str!("../README.md")]
#![forbid(unsafe_code)]

use bevy::{
    ecs::system::Command,
    prelude::{Commands, Component, Deref, DerefMut, Resource, World},
    tasks::IoTaskPool,
};
pub use matchbox_socket_nostr;
use matchbox_socket_nostr::{
    BuildablePlurality, MessageLoopFuture, SingleChannel, WebRtcSocket, WebRtcSocketBuilder,
};
use nostr::Keys;
use std::marker::PhantomData;

/// A [`WebRtcSocket`] as a [`Component`] or [`Resource`].
///
/// As a [`Component`], directly
/// ```
/// use bevy_matchbox::prelude::*;
/// use bevy::prelude::*;
///
/// fn open_socket_system(mut commands: Commands) {
///     let room_url = "wss://matchbox.example.com";
///     let builder = WebRtcSocketBuilder::new(room_url).add_channel(ChannelConfig::reliable());
///     commands.spawn(MatchboxSocket::from(builder));
/// }
///
/// fn close_socket_system(
///     mut commands: Commands,
///     socket: Query<Entity, With<MatchboxSocket<SingleChannel>>>
/// ) {
///     let socket = socket.single();
///     commands.entity(socket).despawn();
/// }
///
/// ```
///
/// As a [`Resource`], with [`Commands`]
/// ```
/// # use bevy_matchbox::prelude::*;
/// # use bevy::prelude::*;
/// fn open_socket_system(mut commands: Commands) {
///     let room_url = "wss://matchbox.example.com";
///     commands.open_socket(WebRtcSocketBuilder::new(room_url).add_channel(ChannelConfig::reliable()));
/// }
///
/// fn close_socket_system(mut commands: Commands) {
///     commands.close_socket::<SingleChannel>();
/// }
/// ```
///
/// As a [`Resource`], directly
///
/// ```
/// # use bevy_matchbox::prelude::*;
/// # use bevy::prelude::*;
/// fn open_socket_system(mut commands: Commands) {
///     let room_url = "wss://matchbox.example.com";
///
///     let socket: MatchboxSocket<SingleChannel> = WebRtcSocket::builder(room_url)
///         .add_channel(ChannelConfig::reliable())
///         .into();
///
///     commands.insert_resource(socket);
/// }
///
/// fn close_socket_system(mut commands: Commands) {
///     commands.remove_resource::<MatchboxSocket<SingleChannel>>();
/// }
/// ```
#[derive(Resource, Component, Debug, Deref, DerefMut)]
pub struct MatchboxSocket<C: BuildablePlurality>(WebRtcSocket<C>);

impl<C: BuildablePlurality> From<WebRtcSocketBuilder<C>> for MatchboxSocket<C> {
    fn from(builder: WebRtcSocketBuilder<C>) -> Self {
        Self::from(builder.build())
    }
}

impl<C: BuildablePlurality> From<(WebRtcSocket<C>, MessageLoopFuture)> for MatchboxSocket<C> {
    fn from((socket, message_loop_fut): (WebRtcSocket<C>, MessageLoopFuture)) -> Self {
        let task_pool = IoTaskPool::get();
        task_pool.spawn(message_loop_fut).detach();
        MatchboxSocket(socket)
    }
}

/// A [`Command`] used to open a [`MatchboxSocket`] and allocate it as a resource.
struct OpenSocket<C: BuildablePlurality>(WebRtcSocketBuilder<C>);

impl<C: BuildablePlurality + 'static> Command for OpenSocket<C> {
    fn write(self, world: &mut World) {
        world.insert_resource(MatchboxSocket::from(self.0));
    }
}

/// A [`Commands`] extension used to open a [`MatchboxSocket`] and allocate it as a resource.
pub trait OpenSocketExt<C: BuildablePlurality> {
    /// Opens a [`MatchboxSocket`] and allocates it as a resource.
    fn open_socket(&mut self, socket_builder: WebRtcSocketBuilder<C>);
}

impl<'w, 's, C: BuildablePlurality + 'static> OpenSocketExt<C> for Commands<'w, 's> {
    fn open_socket(&mut self, socket_builder: WebRtcSocketBuilder<C>) {
        self.add(OpenSocket(socket_builder))
    }
}

/// A [`Command`] used to close a [`WebRtcSocket`], deleting the [`MatchboxSocket`] resource.
struct CloseSocket<C: BuildablePlurality>(PhantomData<C>);

impl<C: BuildablePlurality + 'static> Command for CloseSocket<C> {
    fn write(self, world: &mut World) {
        world.remove_resource::<MatchboxSocket<C>>();
    }
}

/// A [`Commands`] extension used to close a [`WebRtcSocket`], deleting the [`MatchboxSocket`] resource.
pub trait CloseSocketExt {
    /// Delete the [`MatchboxSocket`] resource.
    fn close_socket<C: BuildablePlurality + 'static>(&mut self);
}

impl<'w, 's> CloseSocketExt for Commands<'w, 's> {
    fn close_socket<C: BuildablePlurality + 'static>(&mut self) {
        self.add(CloseSocket::<C>(PhantomData::default()))
    }
}

/// use `bevy_matchbox::prelude::*;` to import common resources and commands
pub mod prelude {
    pub use crate::{CloseSocketExt, MatchboxSocket, OpenSocketExt};
    pub use matchbox_socket_nostr::{
        BuildablePlurality, ChannelConfig, MultipleChannels, PeerId, PeerState, SingleChannel,
        WebRtcSocketBuilder,
    };
}

impl MatchboxSocket<SingleChannel> {
    /// Create a new socket with a single unreliable channel
    ///
    /// ```rust
    /// # use bevy_matchbox::prelude::*;
    /// # use bevy::prelude::*;
    /// # fn open_channel_system(mut commands: Commands) {
    /// let room_url = "wss://matchbox.example.com";
    /// let socket = MatchboxSocket::new_unreliable(room_url);
    /// commands.spawn(socket);
    /// # }
    /// ```
    pub fn new_unreliable(
        room_url: impl Into<String>,
        nostr_keys: Keys,
    ) -> MatchboxSocket<SingleChannel> {
        Self::from(WebRtcSocket::new_unreliable(room_url, nostr_keys))
    }

    /// Create a new socket with a single reliable channel
    ///
    /// ```rust
    /// # use bevy_matchbox::prelude::*;
    /// # use bevy::prelude::*;
    /// # fn open_channel_system(mut commands: Commands) {
    /// let room_url = "wss://matchbox.example.com";
    /// let socket = MatchboxSocket::new_reliable(room_url);
    /// commands.spawn(socket);
    /// # }
    /// ```
    pub fn new_reliable(
        room_url: impl Into<String>,
        nostr_keys: Keys,
    ) -> MatchboxSocket<SingleChannel> {
        Self::from(WebRtcSocket::new_reliable(room_url, nostr_keys))
    }

    /// Create a new socket with a single ggrs-compatible channel
    ///
    /// ```rust
    /// # use bevy_matchbox::prelude::*;
    /// # use bevy::prelude::*;
    /// # fn open_channel_system(mut commands: Commands) {
    /// let room_url = "wss://matchbox.example.com";
    /// let socket = MatchboxSocket::new_ggrs(room_url);
    /// commands.spawn(socket);
    /// # }
    /// ```
    #[cfg(feature = "ggrs")]
    pub fn new_ggrs(
        room_url: impl Into<String>,
        nostr_keys: Keys,
    ) -> MatchboxSocket<SingleChannel> {
        Self::from(WebRtcSocket::new_ggrs(room_url, nostr_keys))
    }
}
