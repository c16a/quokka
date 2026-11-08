//! German. A string not here falls back to American English.

use super::Translation;

pub struct DeDe;

impl Translation for DeDe {
    fn language_tags(&self) -> &'static [&'static str] {
        &["de-DE", "de"]
    }

    fn native_name(&self) -> &'static str {
        "Deutsch"
    }

    // albums.rs
    fn name(&self) -> Option<&'static str> {
        Some("Name")
    }
    fn year(&self) -> Option<&'static str> {
        Some("Jahr")
    }
    fn date_added(&self) -> Option<&'static str> {
        Some("Hinzugefügt")
    }
    fn albums(&self) -> Option<&'static str> {
        Some("Alben")
    }
    fn no_albums_match_these_filters(&self) -> Option<&'static str> {
        Some("Keine Alben entsprechen diesen Filtern.")
    }
    fn no_albums_yet_choose_your_library(&self) -> Option<&'static str> {
        Some("Noch keine Alben. Wähle deinen Mediathek-Ordner in den Einstellungen.")
    }
    fn all_genres(&self) -> Option<&'static str> {
        Some("Alle Genres")
    }
    fn all_years(&self) -> Option<&'static str> {
        Some("Alle Jahre")
    }
    fn all_albums(&self) -> Option<&'static str> {
        Some("Alle Alben")
    }
    fn unknown_album(&self) -> Option<&'static str> {
        Some("Unbekanntes Album")
    }
    fn play(&self) -> Option<&'static str> {
        Some("Wiedergeben")
    }
    fn shuffle(&self) -> Option<&'static str> {
        Some("Zufällig")
    }
    fn remove_from_favourites(&self) -> Option<&'static str> {
        Some("Aus Favoriten entfernen")
    }
    fn add_to_favourites(&self) -> Option<&'static str> {
        Some("Zu Favoriten hinzufügen")
    }
    fn album(&self) -> Option<&'static str> {
        Some("ALBUM")
    }
    // artists.rs
    fn no_artists_yet_choose_your_library(&self) -> Option<&'static str> {
        Some("Noch keine Künstler. Wähle deinen Mediathek-Ordner in den Einstellungen.")
    }
    fn artists(&self) -> Option<&'static str> {
        Some("Künstler")
    }
    fn all_artists(&self) -> Option<&'static str> {
        Some("Alle Künstler")
    }
    // format.rs
    fn unavailable(&self) -> Option<&'static str> {
        Some("Nicht verfügbar")
    }
    fn mono(&self) -> Option<&'static str> {
        Some("Mono")
    }
    fn stereo(&self) -> Option<&'static str> {
        Some("Stereo")
    }
    // lib.rs
    fn add_a_folder_to_get_started(&self) -> Option<&'static str> {
        Some("Füge einen Ordner hinzu, um loszulegen.")
    }
    fn syncing_the_radio_station_list(&self) -> Option<&'static str> {
        Some("Senderliste wird synchronisiert …")
    }
    fn radio_station_sync_stopped_unexpectedly(&self) -> Option<&'static str> {
        Some("Die Synchronisierung der Senderliste wurde unerwartet beendet.")
    }
    fn the_scan_stopped_unexpectedly(&self) -> Option<&'static str> {
        Some("der Scan wurde unerwartet beendet")
    }
    fn video(&self) -> Option<&'static str> {
        Some("Video")
    }
    fn search(&self) -> Option<&'static str> {
        Some("Suche")
    }
    fn recent(&self) -> Option<&'static str> {
        Some("Zuletzt")
    }
    fn playlists(&self) -> Option<&'static str> {
        Some("Playlists")
    }
    fn radio(&self) -> Option<&'static str> {
        Some("Radio")
    }
    fn movies(&self) -> Option<&'static str> {
        Some("Filme")
    }
    fn settings(&self) -> Option<&'static str> {
        Some("Einstellungen")
    }
    fn scanning(&self) -> Option<&'static str> {
        Some("Wird gescannt …")
    }
    fn saved_the_tmdb_key(&self) -> Option<&'static str> {
        Some("TMDb-Schlüssel gespeichert.")
    }
    fn could_not_save_settings(&self) -> Option<&'static str> {
        Some("Einstellungen konnten nicht gespeichert werden: {0}")
    }
    fn radio_station_count(&self) -> Option<&'static str> {
        Some("Radio: {0} Sender.")
    }
    fn scanning_folder(&self) -> Option<&'static str> {
        Some("{0} wird gescannt …")
    }
    fn scanning_folder_queued(&self) -> Option<&'static str> {
        Some("{0} wird gescannt ({1} weitere in der Warteschlange) …")
    }
    fn scan_summary(&self) -> Option<&'static str> {
        Some("{0}: {1} hinzugefügt oder geändert, {2} unverändert, {3} entfernt.")
    }
    fn matched_on_tmdb(&self) -> Option<&'static str> {
        Some("{0} auf TMDb zugeordnet.")
    }
    fn tmdb_lookup_stopped(&self) -> Option<&'static str> {
        Some("TMDb-Suche abgebrochen ({0}); sie wird beim nächsten Scan erneut versucht.")
    }
    fn could_not_find_album(&self) -> Option<&'static str> {
        Some("Album {0} nicht gefunden")
    }
    fn could_not_tune_in(&self) -> Option<&'static str> {
        Some("Sender konnte nicht hinzugefügt werden: {0}")
    }
    fn could_not_tune_out(&self) -> Option<&'static str> {
        Some("Sender konnte nicht entfernt werden: {0}")
    }
    fn could_not_add_to_playlist(&self) -> Option<&'static str> {
        Some("Konnte nicht zur Playlist hinzugefügt werden: {0}")
    }
    fn already_in_library(&self) -> Option<&'static str> {
        Some("{0} ist bereits in der Mediathek.")
    }
    fn removed_folder(&self) -> Option<&'static str> {
        Some("{0} entfernt ({1}).")
    }
    fn could_not_remove_folder(&self) -> Option<&'static str> {
        Some("{0} konnte nicht entfernt werden: {1}")
    }
    fn hide_sidebar(&self) -> Option<&'static str> {
        Some("Seitenleiste ausblenden")
    }
    fn show_sidebar(&self) -> Option<&'static str> {
        Some("Seitenleiste einblenden")
    }
    // movies.rs
    fn no_movies_yet_choose_your_library(&self) -> Option<&'static str> {
        Some("Noch keine Filme. Wähle deinen Mediathek-Ordner in den Einstellungen.")
    }
    fn movie(&self) -> Option<&'static str> {
        Some("FILM")
    }
    fn length_unknown(&self) -> Option<&'static str> {
        Some("Länge unbekannt")
    }
    fn start_over(&self) -> Option<&'static str> {
        Some("Von vorne")
    }
    fn wrong_movie_choose_another(&self) -> Option<&'static str> {
        Some("Falscher Film? Anderen wählen")
    }
    fn remove_details(&self) -> Option<&'static str> {
        Some("Details entfernen")
    }
    fn file_information(&self) -> Option<&'static str> {
        Some("Dateiinformationen")
    }
    fn source(&self) -> Option<&'static str> {
        Some("QUELLE")
    }
    fn finding_details(&self) -> Option<&'static str> {
        Some("Details werden gesucht …")
    }
    fn find_details_on_tmdb(&self) -> Option<&'static str> {
        Some("Details auf TMDb suchen")
    }
    fn add_your_tmdb_api_key_in(&self) -> Option<&'static str> {
        Some(
            "Füge in den Einstellungen deinen TMDb-API-Schlüssel hinzu, um Beschreibung und Besetzung zu sehen.",
        )
    }
    fn synopsis(&self) -> Option<&'static str> {
        Some("Handlung")
    }
    fn cast(&self) -> Option<&'static str> {
        Some("Besetzung")
    }
    fn director(&self) -> Option<&'static str> {
        Some("Regie")
    }
    fn writer(&self) -> Option<&'static str> {
        Some("Drehbuch")
    }
    fn crew(&self) -> Option<&'static str> {
        Some("Stab")
    }
    fn which_movie_is_this(&self) -> Option<&'static str> {
        Some("Welcher Film ist das?")
    }
    fn pick_the_one_your_file_is(&self) -> Option<&'static str> {
        Some("Wähle den Film, der zu deiner Datei passt. Gespeichert wird erst, wenn du wählst.")
    }
    fn none_of_these(&self) -> Option<&'static str> {
        Some("Keiner davon")
    }
    fn left_unmatched(&self) -> Option<&'static str> {
        Some("Nicht zugeordnet.")
    }
    fn this_one(&self) -> Option<&'static str> {
        Some("Dieser")
    }
    fn the_lookup_stopped_unexpectedly(&self) -> Option<&'static str> {
        Some("die Suche wurde unerwartet beendet")
    }
    fn no_tmdb_details_for_this_movie(&self) -> Option<&'static str> {
        Some("Keine TMDb-Details für diesen Film.")
    }
    fn unknown(&self) -> Option<&'static str> {
        Some("Unbekannt")
    }
    fn container(&self) -> Option<&'static str> {
        Some("Container")
    }
    fn codec(&self) -> Option<&'static str> {
        Some("Codec")
    }
    fn colour_depth(&self) -> Option<&'static str> {
        Some("Farbtiefe")
    }
    fn chroma_subsampling(&self) -> Option<&'static str> {
        Some("Chroma-Unterabtastung")
    }
    fn dynamic_range(&self) -> Option<&'static str> {
        Some("Dynamikumfang")
    }
    fn resolution(&self) -> Option<&'static str> {
        Some("Auflösung")
    }
    fn aspect_ratio(&self) -> Option<&'static str> {
        Some("Seitenverhältnis")
    }
    fn frame_rate(&self) -> Option<&'static str> {
        Some("Bildrate")
    }
    fn length(&self) -> Option<&'static str> {
        Some("Länge")
    }
    fn file_size(&self) -> Option<&'static str> {
        Some("Dateigröße")
    }
    fn mpeg_transport_stream(&self) -> Option<&'static str> {
        Some("MPEG-Transportstrom")
    }
    fn mpeg_program_stream(&self) -> Option<&'static str> {
        Some("MPEG-Programmstrom")
    }
    fn raw_av1_stream_obu(&self) -> Option<&'static str> {
        Some("AV1-Rohdatenstrom (OBU)")
    }
    // playback.rs
    fn no_audio_output_device_available(&self) -> Option<&'static str> {
        Some("kein Audioausgabegerät verfügbar")
    }
    fn could_not_connect_to_the_station(&self) -> Option<&'static str> {
        Some("Verbindung zum Sender fehlgeschlagen.")
    }
    fn the_station_keeps_dropping_the_connection(&self) -> Option<&'static str> {
        Some(
            "Der Sender bricht die Verbindung immer wieder ab. Drücke auf Wiedergabe, um es erneut zu versuchen.",
        )
    }
    fn pause(&self) -> Option<&'static str> {
        Some("Pause")
    }
    fn connecting(&self) -> Option<&'static str> {
        Some("Verbindung wird hergestellt …")
    }
    fn paused(&self) -> Option<&'static str> {
        Some("Pausiert")
    }
    fn live(&self) -> Option<&'static str> {
        Some("Live")
    }
    fn no_audio_output_device_available_2(&self) -> Option<&'static str> {
        Some("Kein Audioausgabegerät verfügbar")
    }
    fn back_10_seconds_left_arrow(&self) -> Option<&'static str> {
        Some("10 Sekunden zurück (Pfeil nach links)")
    }
    fn pause_space(&self) -> Option<&'static str> {
        Some("Pause (Leertaste)")
    }
    fn play_space(&self) -> Option<&'static str> {
        Some("Wiedergeben (Leertaste)")
    }
    fn forward_10_seconds_right_arrow(&self) -> Option<&'static str> {
        Some("10 Sekunden vor (Pfeil nach rechts)")
    }
    fn exit_full_screen_escape(&self) -> Option<&'static str> {
        Some("Vollbild beenden (Esc)")
    }
    fn full_screen_f(&self) -> Option<&'static str> {
        Some("Vollbild (F)")
    }
    fn unmute(&self) -> Option<&'static str> {
        Some("Ton an")
    }
    fn mute(&self) -> Option<&'static str> {
        Some("Stumm")
    }
    fn loading_video(&self) -> Option<&'static str> {
        Some("Video wird geladen …")
    }
    fn previous_track(&self) -> Option<&'static str> {
        Some("Vorheriger Titel")
    }
    fn next_track(&self) -> Option<&'static str> {
        Some("Nächster Titel")
    }
    fn repeat(&self) -> Option<&'static str> {
        Some("Wiederholen")
    }
    fn nothing_playing(&self) -> Option<&'static str> {
        Some("Keine Wiedergabe")
    }
    fn output(&self) -> Option<&'static str> {
        Some("Ausgabe")
    }
    fn system_default(&self) -> Option<&'static str> {
        Some("Systemstandard")
    }
    fn up_next(&self) -> Option<&'static str> {
        Some("Als Nächstes")
    }
    fn lyrics(&self) -> Option<&'static str> {
        Some("Liedtext")
    }
    fn playback_speed_click_to_change_speeds(&self) -> Option<&'static str> {
        Some(
            "Wiedergabegeschwindigkeit. Zum Ändern klicken. Andere Geschwindigkeiten als 1× ändern auch die Tonhöhe.",
        )
    }
    fn track_info(&self) -> Option<&'static str> {
        Some("Titelinfo")
    }
    // playlists.rs
    fn new_playlist_name(&self) -> Option<&'static str> {
        Some("Name der neuen Playlist")
    }
    fn create(&self) -> Option<&'static str> {
        Some("Erstellen")
    }
    fn no_playlists_yet_add_tracks_from(&self) -> Option<&'static str> {
        Some("Noch keine Playlists. Füge Titel aus einem Album oder den Suchergebnissen hinzu.")
    }
    fn rename(&self) -> Option<&'static str> {
        Some("Umbenennen")
    }
    fn delete_playlist(&self) -> Option<&'static str> {
        Some("Playlist löschen")
    }
    fn all_playlists(&self) -> Option<&'static str> {
        Some("Alle Playlists")
    }
    fn save(&self) -> Option<&'static str> {
        Some("Sichern")
    }
    fn cancel(&self) -> Option<&'static str> {
        Some("Abbrechen")
    }
    fn play_playlist(&self) -> Option<&'static str> {
        Some("Playlist wiedergeben")
    }
    fn shuffle_playlist(&self) -> Option<&'static str> {
        Some("Playlist zufällig wiedergeben")
    }
    // queue.rs
    fn up_next_2(&self) -> Option<&'static str> {
        Some("Als Nächstes")
    }
    fn nothing_queued(&self) -> Option<&'static str> {
        Some("Nichts in der Warteschlange")
    }
    fn clear(&self) -> Option<&'static str> {
        Some("Leeren")
    }
    fn tracks_you_queue_will_appear_here(&self) -> Option<&'static str> {
        Some("Titel, die du einreihst, erscheinen hier.")
    }
    fn play_next(&self) -> Option<&'static str> {
        Some("Als Nächstes spielen")
    }
    fn add_to_queue(&self) -> Option<&'static str> {
        Some("Zur Warteschlange hinzufügen")
    }
    fn add_to_playlist(&self) -> Option<&'static str> {
        Some("Zur Playlist hinzufügen")
    }
    fn no_playlists_yet(&self) -> Option<&'static str> {
        Some("Noch keine Playlists")
    }
    // radio.rs
    fn no_stations_yet(&self) -> Option<&'static str> {
        Some("Noch keine Sender.")
    }
    fn sync_now(&self) -> Option<&'static str> {
        Some("Jetzt synchronisieren")
    }
    fn syncing(&self) -> Option<&'static str> {
        Some("Wird synchronisiert …")
    }
    fn your_stations(&self) -> Option<&'static str> {
        Some("Deine Sender")
    }
    fn find_a_station_below_and_tune(&self) -> Option<&'static str> {
        Some("Suche unten einen Sender und schalte ein. Er erscheint dann hier.")
    }
    fn find_stations(&self) -> Option<&'static str> {
        Some("Sender suchen")
    }
    fn sync_the_station_list_to_search(&self) -> Option<&'static str> {
        Some("Synchronisiere die Senderliste, um sie zu durchsuchen.")
    }
    fn no_stations_match(&self) -> Option<&'static str> {
        Some("Keine passenden Sender.")
    }
    fn country_any(&self) -> Option<&'static str> {
        Some("Land: alle")
    }
    fn any_country(&self) -> Option<&'static str> {
        Some("Alle Länder")
    }
    fn codec_any(&self) -> Option<&'static str> {
        Some("Codec: alle")
    }
    fn any_codec(&self) -> Option<&'static str> {
        Some("Alle Codecs")
    }
    fn bitrate_any(&self) -> Option<&'static str> {
        Some("Bitrate: alle")
    }
    fn any_bitrate(&self) -> Option<&'static str> {
        Some("Alle Bitraten")
    }
    fn most_popular_first(&self) -> Option<&'static str> {
        Some("Beliebteste zuerst")
    }
    fn paused_2(&self) -> Option<&'static str> {
        Some("PAUSIERT")
    }
    fn live_now(&self) -> Option<&'static str> {
        Some("JETZT LIVE")
    }
    fn tune_out(&self) -> Option<&'static str> {
        Some("Ausschalten")
    }
    fn search_by_name_country_language_or(&self) -> Option<&'static str> {
        Some("Nach Name, Land, Sprache oder Genre suchen")
    }
    fn internet_radio(&self) -> Option<&'static str> {
        Some("Internetradio")
    }
    fn just_now(&self) -> Option<&'static str> {
        Some("gerade eben")
    }
    // recent.rs
    fn recently_added(&self) -> Option<&'static str> {
        Some("Zuletzt hinzugefügt")
    }
    fn albums_you_add_to_the_library(&self) -> Option<&'static str> {
        Some("Alben, die du der Mediathek hinzufügst, erscheinen hier.")
    }
    fn recently_played(&self) -> Option<&'static str> {
        Some("Zuletzt gespielt")
    }
    fn albums_you_play_appear_here(&self) -> Option<&'static str> {
        Some("Alben, die du abspielst, erscheinen hier.")
    }
    // search.rs
    fn search_albums_songs_playlists_and_stations(&self) -> Option<&'static str> {
        Some("Alben, Songs, Playlists und Sender durchsuchen")
    }
    fn search_for_an_album_a_song(&self) -> Option<&'static str> {
        Some(
            "Suche nach einem Album, einem Song, einer Playlist oder einem Sender. Filter: artist:, album:, genre:, year:, format:flac.",
        )
    }
    fn stations(&self) -> Option<&'static str> {
        Some("Sender")
    }
    fn songs(&self) -> Option<&'static str> {
        Some("Songs")
    }
    // settings_view.rs
    fn music(&self) -> Option<&'static str> {
        Some("Musik")
    }
    fn remove(&self) -> Option<&'static str> {
        Some("Entfernen")
    }
    fn rescan(&self) -> Option<&'static str> {
        Some("Neu scannen")
    }
    fn holds(&self) -> Option<&'static str> {
        Some("Enthält:")
    }
    fn library(&self) -> Option<&'static str> {
        Some("Mediathek")
    }
    fn no_folders_yet(&self) -> Option<&'static str> {
        Some("Noch keine Ordner.")
    }
    fn rescan_all(&self) -> Option<&'static str> {
        Some("Alle neu scannen")
    }
    fn processing(&self) -> Option<&'static str> {
        Some("Wird verarbeitet")
    }
    fn kind_not_set(&self) -> Option<&'static str> {
        Some("Art nicht festgelegt")
    }
    fn choose_what_each_of_these_folders(&self) -> Option<&'static str> {
        Some("Wähle, was jeder dieser Ordner enthält.")
    }
    fn tv_shows(&self) -> Option<&'static str> {
        Some("Serien")
    }
    fn coming_soon(&self) -> Option<&'static str> {
        Some("Demnächst")
    }
    fn add_a_folder(&self) -> Option<&'static str> {
        Some("Ordner hinzufügen")
    }
    fn each_folder_holds_one_kind_of(&self) -> Option<&'static str> {
        Some(
            "Jeder Ordner enthält eine Art von Medien. Andere Dateiarten darin werden übersprungen.",
        )
    }
    fn tv_shows_soon(&self) -> Option<&'static str> {
        Some("Serien · Demnächst")
    }
    fn choose_folder(&self) -> Option<&'static str> {
        Some("Ordner wählen …")
    }
    fn choose_a_folder_for_your_library(&self) -> Option<&'static str> {
        Some("Wähle einen Ordner für deine Mediathek")
    }
    fn appearance(&self) -> Option<&'static str> {
        Some("Darstellung")
    }
    fn theme(&self) -> Option<&'static str> {
        Some("Design")
    }
    fn language(&self) -> Option<&'static str> {
        Some("Sprache")
    }
    fn language_system(&self) -> Option<&'static str> {
        Some("System ({0})")
    }
    fn dark(&self) -> Option<&'static str> {
        Some("Dunkel")
    }
    fn light(&self) -> Option<&'static str> {
        Some("Hell")
    }
    fn system(&self) -> Option<&'static str> {
        Some("System")
    }
    fn playback(&self) -> Option<&'static str> {
        Some("Wiedergabe")
    }
    fn loudness(&self) -> Option<&'static str> {
        Some("Lautstärkeausgleich")
    }
    fn levels_tracks_to_the_same_loudness(&self) -> Option<&'static str> {
        Some(
            "Gleicht Titel auf dieselbe Lautheit an. Dateien mit ReplayGain-Tags nutzen diese; andere Dateien werden beim Abspielen im Hintergrund gemessen und behalten bis dahin ihren eigenen Pegel.",
        )
    }
    fn album_2(&self) -> Option<&'static str> {
        Some("Album")
    }
    fn track(&self) -> Option<&'static str> {
        Some("Titel")
    }
    fn off(&self) -> Option<&'static str> {
        Some("Aus")
    }
    fn crossfade(&self) -> Option<&'static str> {
        Some("Überblenden")
    }
    fn each_track_fades_into_the_next(&self) -> Option<&'static str> {
        Some(
            "Jeder Titel blendet über so viele Sekunden in den nächsten über. Null schaltet es aus.",
        )
    }
    fn look_up_synced_lyrics_on_lrclib(&self) -> Option<&'static str> {
        Some("Synchronisierte Liedtexte auf lrclib.net suchen")
    }
    fn for_tracks_with_no_lrc_file(&self) -> Option<&'static str> {
        Some(
            "Für Titel ohne .lrc-Datei daneben. Sendet Titel, Künstler, Album und Länge jedes Titels an lrclib.net. Gefundene Liedtexte werden in der Mediathek zwischengespeichert. Eine .lrc-Datei hat immer Vorrang.",
        )
    }
    fn movie_details(&self) -> Option<&'static str> {
        Some("Filmdetails")
    }
    fn descriptions_and_cast_for_movies_come(&self) -> Option<&'static str> {
        Some(
            "Beschreibungen und Besetzung von Filmen stammen von TMDb. Gib deinen eigenen TMDb-API-Schlüssel ein, um sie zu nutzen. Der Schlüssel wird in deiner Einstellungsdatei gespeichert, ist nur für dein Benutzerkonto lesbar und wird nur an TMDb gesendet.",
        )
    }
    fn tmdb_api_key(&self) -> Option<&'static str> {
        Some("TMDb-API-Schlüssel")
    }
    fn save_key(&self) -> Option<&'static str> {
        Some("Schlüssel sichern")
    }
    fn remove_key(&self) -> Option<&'static str> {
        Some("Schlüssel entfernen")
    }
    fn a_key_is_saved(&self) -> Option<&'static str> {
        Some("Ein Schlüssel ist gespeichert.")
    }
    fn no_key_saved_movies_show_no(&self) -> Option<&'static str> {
        Some(
            "Kein Schlüssel gespeichert. Filme zeigen keine Beschreibungen oder Besetzung, bis du einen hinzufügst.",
        )
    }
    fn this_product_uses_the_tmdb_api(&self) -> Option<&'static str> {
        Some(
            "Dieses Produkt nutzt die TMDB-API, ist aber nicht von TMDB unterstützt oder zertifiziert.",
        )
    }
    // side.rs
    fn nothing_playing_2(&self) -> Option<&'static str> {
        Some("Keine Wiedergabe.")
    }
    fn this_track_has_no_lyrics_in(&self) -> Option<&'static str> {
        Some("Dieser Titel hat keinen Liedtext in seinen Tags und keine .lrc-Datei daneben.")
    }
    fn looking_up_lyrics_on_lrclib_net(&self) -> Option<&'static str> {
        Some("Liedtext wird auf lrclib.net gesucht …")
    }
    fn no_lyrics_were_found_on_lrclib(&self) -> Option<&'static str> {
        Some("Auf lrclib.net wurde kein Liedtext für diesen Titel gefunden.")
    }
    fn try_again(&self) -> Option<&'static str> {
        Some("Erneut versuchen")
    }
    fn title(&self) -> Option<&'static str> {
        Some("Titel")
    }
    fn artist(&self) -> Option<&'static str> {
        Some("Künstler")
    }
    fn duration(&self) -> Option<&'static str> {
        Some("Dauer")
    }
    fn size(&self) -> Option<&'static str> {
        Some("Größe")
    }
    fn bitrate(&self) -> Option<&'static str> {
        Some("Bitrate")
    }
    fn format(&self) -> Option<&'static str> {
        Some("Format")
    }
    fn channels(&self) -> Option<&'static str> {
        Some("Kanäle")
    }
    fn pictures(&self) -> Option<&'static str> {
        Some("Bilder")
    }
    fn file(&self) -> Option<&'static str> {
        Some("Datei")
    }
    fn reveal_in_finder(&self) -> Option<&'static str> {
        Some("Im Finder anzeigen")
    }
    fn tags(&self) -> Option<&'static str> {
        Some("Tags")
    }
    fn revert(&self) -> Option<&'static str> {
        Some("Zurücksetzen")
    }
    fn put_back_the_tags_the_file(&self) -> Option<&'static str> {
        Some("Die Tags wiederherstellen, die die Datei vor der ersten Bearbeitung hatte")
    }
    fn look_up_synced_lyrics_on_lrclib_2(&self) -> Option<&'static str> {
        Some(
            "Synchronisierte Liedtexte auf lrclib.net suchen? Titel, Künstler, Album und Länge jedes geöffneten Titels werden dorthin gesendet.",
        )
    }
    fn turn_on_lyrics_lookup(&self) -> Option<&'static str> {
        Some("Liedtextsuche einschalten")
    }
    fn photos_credits(&self) -> Option<&'static str> {
        Some("Fotos: {0}")
    }
    fn duration_hours_minutes(&self) -> Option<&'static str> {
        Some("{0} Std. {1} Min.")
    }
    fn duration_minutes(&self) -> Option<&'static str> {
        Some("{0} Min.")
    }
    fn resume_at(&self) -> Option<&'static str> {
        Some("Fortsetzen bei {0}")
    }
    fn could_not_fetch_details(&self) -> Option<&'static str> {
        Some("Details konnten nicht geladen werden: {0}")
    }
    fn fps_average_variable(&self) -> Option<&'static str> {
        Some("{0} fps im Mittel, variabel")
    }
    #[cfg(not(target_os = "macos"))]
    fn video_needs_libmpv(&self) -> Option<&'static str> {
        Some("Video benötigt libmpv: {0}")
    }
    fn no_audio_output_error(&self) -> Option<&'static str> {
        Some("Keine Audioausgabe: {0}")
    }
    fn could_not_play_station(&self) -> Option<&'static str> {
        Some("Sender konnte nicht wiedergegeben werden: {0}")
    }
    fn could_not_connect_station_error(&self) -> Option<&'static str> {
        Some("Verbindung zum Sender fehlgeschlagen: {0}")
    }
    fn could_not_switch_output(&self) -> Option<&'static str> {
        Some("Ausgabe konnte nicht gewechselt werden: {0}")
    }
    fn could_not_play_video(&self) -> Option<&'static str> {
        Some("Video konnte nicht wiedergegeben werden: {0}")
    }
    fn playback_stopped_early(&self) -> Option<&'static str> {
        Some("Wiedergabe vorzeitig beendet: {0}")
    }
    fn could_not_play_item(&self) -> Option<&'static str> {
        Some("{0} konnte nicht wiedergegeben werden: {1}")
    }
    fn stations_in_directory(&self) -> Option<&'static str> {
        Some("{0} Sender im Verzeichnis · synchronisiert {1}")
    }
    fn tuned_in(&self) -> Option<&'static str> {
        Some("{0} eingeschaltet")
    }
    fn tuned_in_suffix(&self) -> Option<&'static str> {
        Some("  ·  Eingeschaltet")
    }
    fn bitrate_at_least(&self) -> Option<&'static str> {
        Some("Bitrate: ab {0} kbps")
    }
    fn never(&self) -> Option<&'static str> {
        Some("nie")
    }
    fn minutes_ago(&self) -> Option<&'static str> {
        Some("vor {0} Minuten")
    }
    fn hours_ago(&self) -> Option<&'static str> {
        Some("vor {0} Stunden")
    }
    fn days_ago(&self) -> Option<&'static str> {
        Some("vor {0} Tagen")
    }
    fn no_results_for(&self) -> Option<&'static str> {
        Some("Keine Ergebnisse für „{0}“.")
    }
    fn couldnt_reach_lrclib(&self) -> Option<&'static str> {
        Some("lrclib.net nicht erreichbar: {0}")
    }
    fn radio_sync_failed(&self) -> Option<&'static str> {
        Some("Synchronisierung der Senderliste fehlgeschlagen: {0}")
    }
    fn could_not_be_scanned(&self) -> Option<&'static str> {
        Some("{0}: konnte nicht gescannt werden ({1}).")
    }
    fn one_movie(&self) -> Option<&'static str> {
        Some("{0} Film")
    }
    fn many_movies(&self) -> Option<&'static str> {
        Some("{0} Filme")
    }
    fn one_album(&self) -> Option<&'static str> {
        Some("{0} Album")
    }
    fn many_albums(&self) -> Option<&'static str> {
        Some("{0} Alben")
    }
    fn one_track(&self) -> Option<&'static str> {
        Some("{0} Titel")
    }
    fn many_tracks(&self) -> Option<&'static str> {
        Some("{0} Titel")
    }
    fn one_folder(&self) -> Option<&'static str> {
        Some("{0} Ordner")
    }
    fn many_folders(&self) -> Option<&'static str> {
        Some("{0} Ordner")
    }
    fn one_file(&self) -> Option<&'static str> {
        Some("{0} Datei")
    }
    fn many_files(&self) -> Option<&'static str> {
        Some("{0} Dateien")
    }
}
