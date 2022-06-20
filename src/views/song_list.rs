use std::{sync::{Arc, RwLock}, future::ready};

use iced::{Command, pure::{Element, widget::{Column, Text, Button, Rule, Row, Image, button}}, image::Handle, Space, Length, Alignment, alignment::Horizontal};
use crate::{library::{Library, Song}, Message, ui_util::{ElementContainerExtensions, ButtonExtensions}};

use super::content::ContentMessage;

#[derive(Debug, Clone)]
pub enum SongListMessage {
    RefreshSongList,
    RestoreOriginal(Song),
}

impl From<SongListMessage> for Message {
    fn from(slm: SongListMessage) -> Self { ContentMessage::SongListMessage(slm).into() }
}

pub struct SongListView {
    library: Arc<RwLock<Library>>,
    song_views: Vec<(Song, SongView)>,
}

impl SongListView {
    pub fn new(library: Arc<RwLock<Library>>) -> Self {        
        let mut song_views = vec![];
        Self::rebuild_song_views(library.clone(), &mut song_views);
        
        Self { library, song_views }
    }

    pub fn view(&self) -> Element<Message> {
        Column::new()
            .push(Column::with_children(
                self.song_views.iter().map(|x| Some(x)).intersperse_with(|| None).map(|view|
                    if let Some((_, view)) = view {
                        view.view().into()
                    } else {
                        Rule::horizontal(10).into()
                    }
                ).collect()
            ))
            .push(
                Button::new(Text::new("Reload song list"))
                    .on_press(SongListMessage::RefreshSongList.into())
            )
            .into()
    }

    pub fn update(&mut self, message: SongListMessage) -> Command<Message> {
        match message {
            SongListMessage::RefreshSongList => {
                // The content view does this for us!
                Command::perform(ready(()), |_| ContentMessage::OpenSongList.into())
            }

            SongListMessage::RestoreOriginal(song) => {
                song.restore_original_copy().unwrap();
                Command::perform(ready(()), |_| SongListMessage::RefreshSongList.into())
            } 
        }
    }

    fn rebuild_song_views(library: Arc<RwLock<Library>>, views: &mut Vec<(Song, SongView)>) {
        views.clear();

        let library_reader = library.read().unwrap();
        let songs = library_reader.songs();

        for song in songs {
            views.push((song.clone(), SongView::new(library.clone(), song.clone())))
        }
    }
}

struct SongView {
    library: Arc<RwLock<Library>>,
    song: Song,
}

impl SongView {
    pub fn new(library: Arc<RwLock<Library>>, song: Song) -> Self {
        Self {
            library,
            song,
        }
    }

    pub fn view(&self) -> Element<Message> {
        Row::new()
            .padding(10)
            .spacing(10)
            .align_items(Alignment::Center)
            .push_if_let(&self.song.metadata.album_art, |art|
                Image::new(Handle::from_memory(art.data.clone()))
                    .width(Length::Units(100))
            )
            .push(
                Column::new()
                    .push(Text::new(self.song.metadata.title.clone()))
                    .push(Text::new(self.song.metadata.artist.clone()).color([0.3, 0.3, 0.3]))
            )
            .push(Space::with_width(Length::Fill))
            .push(
                Column::new()
                    .spacing(1)
                    .width(Length::Units(140))
                    .push(
                        Button::new(Text::new(if self.song.is_modified() { "Restore original" } else { "Unmodified" }).horizontal_alignment(Horizontal::Center).width(Length::Fill))
                            .on_press_if(self.song.is_modified(), SongListMessage::RestoreOriginal(self.song.clone()).into())
                            .width(Length::Fill)
                    )
                    .push(Button::new(Text::new("Edit metadata").horizontal_alignment(Horizontal::Center).width(Length::Fill))
                        .on_press(ContentMessage::OpenEditMetadata(self.song.clone()).into())
                        .width(Length::Fill)
                    )
                    .push(
                        Button::new(Text::new(if self.song.metadata.is_cropped { "Cropped" } else { "Crop" }).width(Length::Fill).horizontal_alignment(Horizontal::Center))
                            .on_press_if(!self.song.metadata.is_cropped, ContentMessage::OpenCrop(self.song.clone()).into())
                            .width(Length::Fill)
                    )
            )
            .into()
    }
}