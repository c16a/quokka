//! Dutch. A string not here falls back to American English.

use super::Translation;

pub struct NlNl;

impl Translation for NlNl {
    fn language_tags(&self) -> &'static [&'static str] {
        &["nl-NL", "nl"]
    }

    fn native_name(&self) -> &'static str {
        "Nederlands"
    }

    // albums.rs
    fn name(&self) -> Option<&'static str> {
        Some("Naam")
    }
    fn year(&self) -> Option<&'static str> {
        Some("Jaar")
    }
    fn date_added(&self) -> Option<&'static str> {
        Some("Datum toegevoegd")
    }
    fn albums(&self) -> Option<&'static str> {
        Some("Albums")
    }
    fn no_albums_match_these_filters(&self) -> Option<&'static str> {
        Some("Geen albums komen overeen met deze filters.")
    }
    fn no_albums_yet_choose_your_library(&self) -> Option<&'static str> {
        Some("Nog geen albums. Kies je bibliotheekmap in Instellingen.")
    }
    fn all_genres(&self) -> Option<&'static str> {
        Some("Alle genres")
    }
    fn all_years(&self) -> Option<&'static str> {
        Some("Alle jaren")
    }
    fn all_albums(&self) -> Option<&'static str> {
        Some("Alle albums")
    }
    fn unknown_album(&self) -> Option<&'static str> {
        Some("Onbekend album")
    }
    fn play(&self) -> Option<&'static str> {
        Some("Speel af")
    }
    fn shuffle(&self) -> Option<&'static str> {
        Some("Shuffle")
    }
    fn remove_from_favourites(&self) -> Option<&'static str> {
        Some("Verwijder uit favorieten")
    }
    fn add_to_favourites(&self) -> Option<&'static str> {
        Some("Voeg toe aan favorieten")
    }
    fn album(&self) -> Option<&'static str> {
        Some("ALBUM")
    }
    // artists.rs
    fn no_artists_yet_choose_your_library(&self) -> Option<&'static str> {
        Some("Nog geen artiesten. Kies je bibliotheekmap in Instellingen.")
    }
    fn artists(&self) -> Option<&'static str> {
        Some("Artiesten")
    }
    fn all_artists(&self) -> Option<&'static str> {
        Some("Alle artiesten")
    }
    // format.rs
    fn unavailable(&self) -> Option<&'static str> {
        Some("Niet beschikbaar")
    }
    fn mono(&self) -> Option<&'static str> {
        Some("Mono")
    }
    fn stereo(&self) -> Option<&'static str> {
        Some("Stereo")
    }
    // lib.rs
    fn add_a_folder_to_get_started(&self) -> Option<&'static str> {
        Some("Voeg een map toe om te beginnen.")
    }
    fn syncing_the_radio_station_list(&self) -> Option<&'static str> {
        Some("Zenderlijst synchroniseren…")
    }
    fn radio_station_sync_stopped_unexpectedly(&self) -> Option<&'static str> {
        Some("Synchronisatie van de zenderlijst is onverwacht gestopt.")
    }
    fn the_scan_stopped_unexpectedly(&self) -> Option<&'static str> {
        Some("de scan is onverwacht gestopt")
    }
    fn video(&self) -> Option<&'static str> {
        Some("Video")
    }
    fn search(&self) -> Option<&'static str> {
        Some("Zoek")
    }
    fn recent(&self) -> Option<&'static str> {
        Some("Recent")
    }
    fn playlists(&self) -> Option<&'static str> {
        Some("Afspeellijsten")
    }
    fn radio(&self) -> Option<&'static str> {
        Some("Radio")
    }
    fn movies(&self) -> Option<&'static str> {
        Some("Films")
    }
    fn settings(&self) -> Option<&'static str> {
        Some("Instellingen")
    }
    fn scanning(&self) -> Option<&'static str> {
        Some("Scannen…")
    }
    fn saved_the_tmdb_key(&self) -> Option<&'static str> {
        Some("TMDb-sleutel bewaard.")
    }
    fn could_not_save_settings(&self) -> Option<&'static str> {
        Some("Kan instellingen niet bewaren: {0}")
    }
    fn radio_station_count(&self) -> Option<&'static str> {
        Some("Radio: {0} zenders.")
    }
    fn scanning_folder(&self) -> Option<&'static str> {
        Some("{0} scannen…")
    }
    fn scanning_folder_queued(&self) -> Option<&'static str> {
        Some("{0} scannen ({1} meer in de wachtrij)…")
    }
    fn scan_summary(&self) -> Option<&'static str> {
        Some("{0}: {1} toegevoegd of gewijzigd, {2} ongewijzigd, {3} verwijderd.")
    }
    fn matched_on_tmdb(&self) -> Option<&'static str> {
        Some("{0} gekoppeld via TMDb.")
    }
    fn tmdb_lookup_stopped(&self) -> Option<&'static str> {
        Some("Opzoeken op TMDb gestopt ({0}); het wordt bij de volgende scan opnieuw geprobeerd.")
    }
    fn could_not_find_album(&self) -> Option<&'static str> {
        Some("Kan het album {0} niet vinden")
    }
    fn could_not_tune_in(&self) -> Option<&'static str> {
        Some("Kan niet afstemmen: {0}")
    }
    fn could_not_tune_out(&self) -> Option<&'static str> {
        Some("Kan niet stoppen met luisteren: {0}")
    }
    fn could_not_add_to_playlist(&self) -> Option<&'static str> {
        Some("Kan niet toevoegen aan afspeellijst: {0}")
    }
    fn already_in_library(&self) -> Option<&'static str> {
        Some("{0} staat al in de bibliotheek.")
    }
    fn removed_folder(&self) -> Option<&'static str> {
        Some("{0} verwijderd ({1}).")
    }
    fn could_not_remove_folder(&self) -> Option<&'static str> {
        Some("Kan {0} niet verwijderen: {1}")
    }
    fn hide_sidebar(&self) -> Option<&'static str> {
        Some("Verberg navigatiekolom")
    }
    fn show_sidebar(&self) -> Option<&'static str> {
        Some("Toon navigatiekolom")
    }
    // movies.rs
    fn no_movies_yet_choose_your_library(&self) -> Option<&'static str> {
        Some("Nog geen films. Kies je bibliotheekmap in Instellingen.")
    }
    fn movie(&self) -> Option<&'static str> {
        Some("FILM")
    }
    fn length_unknown(&self) -> Option<&'static str> {
        Some("Duur onbekend")
    }
    fn start_over(&self) -> Option<&'static str> {
        Some("Begin opnieuw")
    }
    fn wrong_movie_choose_another(&self) -> Option<&'static str> {
        Some("Verkeerde film? Kies een andere")
    }
    fn remove_details(&self) -> Option<&'static str> {
        Some("Verwijder details")
    }
    fn file_information(&self) -> Option<&'static str> {
        Some("Bestandsinformatie")
    }
    fn source(&self) -> Option<&'static str> {
        Some("BRON")
    }
    fn finding_details(&self) -> Option<&'static str> {
        Some("Details zoeken…")
    }
    fn find_details_on_tmdb(&self) -> Option<&'static str> {
        Some("Zoek details op TMDb")
    }
    fn add_your_tmdb_api_key_in(&self) -> Option<&'static str> {
        Some("Voeg je TMDb-API-sleutel toe in Instellingen om de beschrijving en cast te zien.")
    }
    fn synopsis(&self) -> Option<&'static str> {
        Some("Verhaal")
    }
    fn cast(&self) -> Option<&'static str> {
        Some("Cast")
    }
    fn director(&self) -> Option<&'static str> {
        Some("Regie")
    }
    fn writer(&self) -> Option<&'static str> {
        Some("Scenario")
    }
    fn crew(&self) -> Option<&'static str> {
        Some("Crew")
    }
    fn which_movie_is_this(&self) -> Option<&'static str> {
        Some("Welke film is dit?")
    }
    fn pick_the_one_your_file_is(&self) -> Option<&'static str> {
        Some("Kies de film die bij je bestand hoort. Er wordt niets bewaard tot je kiest.")
    }
    fn none_of_these(&self) -> Option<&'static str> {
        Some("Geen van deze")
    }
    fn left_unmatched(&self) -> Option<&'static str> {
        Some("Niet gekoppeld.")
    }
    fn this_one(&self) -> Option<&'static str> {
        Some("Deze")
    }
    fn the_lookup_stopped_unexpectedly(&self) -> Option<&'static str> {
        Some("het opzoeken is onverwacht gestopt")
    }
    fn no_tmdb_details_for_this_movie(&self) -> Option<&'static str> {
        Some("Geen TMDb-details voor deze film.")
    }
    fn unknown(&self) -> Option<&'static str> {
        Some("Onbekend")
    }
    fn container(&self) -> Option<&'static str> {
        Some("Container")
    }
    fn codec(&self) -> Option<&'static str> {
        Some("Codec")
    }
    fn colour_depth(&self) -> Option<&'static str> {
        Some("Kleurdiepte")
    }
    fn chroma_subsampling(&self) -> Option<&'static str> {
        Some("Chroma-subsampling")
    }
    fn dynamic_range(&self) -> Option<&'static str> {
        Some("Dynamisch bereik")
    }
    fn resolution(&self) -> Option<&'static str> {
        Some("Resolutie")
    }
    fn aspect_ratio(&self) -> Option<&'static str> {
        Some("Beeldverhouding")
    }
    fn frame_rate(&self) -> Option<&'static str> {
        Some("Beeldsnelheid")
    }
    fn length(&self) -> Option<&'static str> {
        Some("Duur")
    }
    fn file_size(&self) -> Option<&'static str> {
        Some("Bestandsgrootte")
    }
    fn mpeg_transport_stream(&self) -> Option<&'static str> {
        Some("MPEG-transportstream")
    }
    fn mpeg_program_stream(&self) -> Option<&'static str> {
        Some("MPEG-programmastream")
    }
    fn raw_av1_stream_obu(&self) -> Option<&'static str> {
        Some("Ruwe AV1-stream (OBU)")
    }
    // playback.rs
    fn no_audio_output_device_available(&self) -> Option<&'static str> {
        Some("geen audio-uitvoerapparaat beschikbaar")
    }
    fn could_not_connect_to_the_station(&self) -> Option<&'static str> {
        Some("Kan geen verbinding maken met de zender.")
    }
    fn the_station_keeps_dropping_the_connection(&self) -> Option<&'static str> {
        Some(
            "De zender verbreekt steeds de verbinding. Druk op afspelen om het opnieuw te proberen.",
        )
    }
    fn pause(&self) -> Option<&'static str> {
        Some("Pauzeer")
    }
    fn connecting(&self) -> Option<&'static str> {
        Some("Verbinden…")
    }
    fn paused(&self) -> Option<&'static str> {
        Some("Gepauzeerd")
    }
    fn live(&self) -> Option<&'static str> {
        Some("Live")
    }
    fn no_audio_output_device_available_2(&self) -> Option<&'static str> {
        Some("Geen audio-uitvoerapparaat beschikbaar")
    }
    fn back_10_seconds_left_arrow(&self) -> Option<&'static str> {
        Some("10 seconden terug (pijl-links)")
    }
    fn pause_space(&self) -> Option<&'static str> {
        Some("Pauzeer (spatiebalk)")
    }
    fn play_space(&self) -> Option<&'static str> {
        Some("Speel af (spatiebalk)")
    }
    fn forward_10_seconds_right_arrow(&self) -> Option<&'static str> {
        Some("10 seconden vooruit (pijl-rechts)")
    }
    fn exit_full_screen_escape(&self) -> Option<&'static str> {
        Some("Verlaat schermvullende weergave (Esc)")
    }
    fn full_screen_f(&self) -> Option<&'static str> {
        Some("Schermvullend (F)")
    }
    fn unmute(&self) -> Option<&'static str> {
        Some("Geluid aan")
    }
    fn mute(&self) -> Option<&'static str> {
        Some("Geluid uit")
    }
    fn loading_video(&self) -> Option<&'static str> {
        Some("Video laden…")
    }
    fn previous_track(&self) -> Option<&'static str> {
        Some("Vorig nummer")
    }
    fn next_track(&self) -> Option<&'static str> {
        Some("Volgend nummer")
    }
    fn repeat(&self) -> Option<&'static str> {
        Some("Herhaal")
    }
    fn nothing_playing(&self) -> Option<&'static str> {
        Some("Er wordt niets afgespeeld")
    }
    fn output(&self) -> Option<&'static str> {
        Some("Uitvoer")
    }
    fn system_default(&self) -> Option<&'static str> {
        Some("Systeemstandaard")
    }
    fn up_next(&self) -> Option<&'static str> {
        Some("Hierna")
    }
    fn lyrics(&self) -> Option<&'static str> {
        Some("Songtekst")
    }
    fn playback_speed_click_to_change_speeds(&self) -> Option<&'static str> {
        Some(
            "Afspeelsnelheid. Klik om te wijzigen. Andere snelheden dan 1× veranderen ook de toonhoogte.",
        )
    }
    fn track_info(&self) -> Option<&'static str> {
        Some("Nummerinfo")
    }
    // playlists.rs
    fn new_playlist_name(&self) -> Option<&'static str> {
        Some("Naam nieuwe afspeellijst")
    }
    fn create(&self) -> Option<&'static str> {
        Some("Maak aan")
    }
    fn no_playlists_yet_add_tracks_from(&self) -> Option<&'static str> {
        Some("Nog geen afspeellijsten. Voeg nummers toe vanuit een album of de zoekresultaten.")
    }
    fn rename(&self) -> Option<&'static str> {
        Some("Wijzig naam")
    }
    fn delete_playlist(&self) -> Option<&'static str> {
        Some("Verwijder afspeellijst")
    }
    fn all_playlists(&self) -> Option<&'static str> {
        Some("Alle afspeellijsten")
    }
    fn save(&self) -> Option<&'static str> {
        Some("Bewaar")
    }
    fn cancel(&self) -> Option<&'static str> {
        Some("Annuleer")
    }
    fn play_playlist(&self) -> Option<&'static str> {
        Some("Speel afspeellijst af")
    }
    fn shuffle_playlist(&self) -> Option<&'static str> {
        Some("Shuffle afspeellijst")
    }
    // queue.rs
    fn up_next_2(&self) -> Option<&'static str> {
        Some("Hierna")
    }
    fn nothing_queued(&self) -> Option<&'static str> {
        Some("Niets in de wachtrij")
    }
    fn clear(&self) -> Option<&'static str> {
        Some("Wis")
    }
    fn tracks_you_queue_will_appear_here(&self) -> Option<&'static str> {
        Some("Nummers die je in de wachtrij zet, verschijnen hier.")
    }
    fn play_next(&self) -> Option<&'static str> {
        Some("Speel hierna af")
    }
    fn add_to_queue(&self) -> Option<&'static str> {
        Some("Voeg toe aan wachtrij")
    }
    fn add_to_playlist(&self) -> Option<&'static str> {
        Some("Voeg toe aan afspeellijst")
    }
    fn no_playlists_yet(&self) -> Option<&'static str> {
        Some("Nog geen afspeellijsten")
    }
    // radio.rs
    fn no_stations_yet(&self) -> Option<&'static str> {
        Some("Nog geen zenders.")
    }
    fn sync_now(&self) -> Option<&'static str> {
        Some("Synchroniseer nu")
    }
    fn syncing(&self) -> Option<&'static str> {
        Some("Synchroniseren…")
    }
    fn your_stations(&self) -> Option<&'static str> {
        Some("Jouw zenders")
    }
    fn find_a_station_below_and_tune(&self) -> Option<&'static str> {
        Some("Zoek hieronder een zender en stem af. Hij verschijnt dan hier.")
    }
    fn find_stations(&self) -> Option<&'static str> {
        Some("Zoek zenders")
    }
    fn sync_the_station_list_to_search(&self) -> Option<&'static str> {
        Some("Synchroniseer de zenderlijst om erin te zoeken.")
    }
    fn no_stations_match(&self) -> Option<&'static str> {
        Some("Geen zenders gevonden.")
    }
    fn country_any(&self) -> Option<&'static str> {
        Some("Land: alle")
    }
    fn any_country(&self) -> Option<&'static str> {
        Some("Alle landen")
    }
    fn codec_any(&self) -> Option<&'static str> {
        Some("Codec: alle")
    }
    fn any_codec(&self) -> Option<&'static str> {
        Some("Alle codecs")
    }
    fn bitrate_any(&self) -> Option<&'static str> {
        Some("Bitsnelheid: alle")
    }
    fn any_bitrate(&self) -> Option<&'static str> {
        Some("Alle bitsnelheden")
    }
    fn most_popular_first(&self) -> Option<&'static str> {
        Some("Populairste eerst")
    }
    fn paused_2(&self) -> Option<&'static str> {
        Some("GEPAUZEERD")
    }
    fn live_now(&self) -> Option<&'static str> {
        Some("NU LIVE")
    }
    fn tune_out(&self) -> Option<&'static str> {
        Some("Stop met luisteren")
    }
    fn search_by_name_country_language_or(&self) -> Option<&'static str> {
        Some("Zoek op naam, land, taal of genre")
    }
    fn internet_radio(&self) -> Option<&'static str> {
        Some("Internetradio")
    }
    fn just_now(&self) -> Option<&'static str> {
        Some("zojuist")
    }
    // recent.rs
    fn recently_added(&self) -> Option<&'static str> {
        Some("Recent toegevoegd")
    }
    fn albums_you_add_to_the_library(&self) -> Option<&'static str> {
        Some("Albums die je aan de bibliotheek toevoegt, verschijnen hier.")
    }
    fn recently_played(&self) -> Option<&'static str> {
        Some("Recent afgespeeld")
    }
    fn albums_you_play_appear_here(&self) -> Option<&'static str> {
        Some("Albums die je afspeelt, verschijnen hier.")
    }
    // search.rs
    fn search_albums_songs_playlists_and_stations(&self) -> Option<&'static str> {
        Some("Zoek in albums, nummers, afspeellijsten en zenders")
    }
    fn search_for_an_album_a_song(&self) -> Option<&'static str> {
        Some(
            "Zoek naar een album, nummer, afspeellijst of zender. Filters: artist:, album:, genre:, year:, format:flac.",
        )
    }
    fn stations(&self) -> Option<&'static str> {
        Some("Zenders")
    }
    fn songs(&self) -> Option<&'static str> {
        Some("Nummers")
    }
    // settings_view.rs
    fn music(&self) -> Option<&'static str> {
        Some("Muziek")
    }
    fn remove(&self) -> Option<&'static str> {
        Some("Verwijder")
    }
    fn rescan(&self) -> Option<&'static str> {
        Some("Scan opnieuw")
    }
    fn holds(&self) -> Option<&'static str> {
        Some("Bevat:")
    }
    fn library(&self) -> Option<&'static str> {
        Some("Bibliotheek")
    }
    fn no_folders_yet(&self) -> Option<&'static str> {
        Some("Nog geen mappen.")
    }
    fn rescan_all(&self) -> Option<&'static str> {
        Some("Scan alles opnieuw")
    }
    fn processing(&self) -> Option<&'static str> {
        Some("Verwerken")
    }
    fn kind_not_set(&self) -> Option<&'static str> {
        Some("Soort niet ingesteld")
    }
    fn choose_what_each_of_these_folders(&self) -> Option<&'static str> {
        Some("Kies wat elk van deze mappen bevat.")
    }
    fn tv_shows(&self) -> Option<&'static str> {
        Some("Tv-series")
    }
    fn coming_soon(&self) -> Option<&'static str> {
        Some("Binnenkort")
    }
    fn add_a_folder(&self) -> Option<&'static str> {
        Some("Voeg een map toe")
    }
    fn each_folder_holds_one_kind_of(&self) -> Option<&'static str> {
        Some("Elke map bevat één soort media. Andere soorten bestanden erin worden overgeslagen.")
    }
    fn tv_shows_soon(&self) -> Option<&'static str> {
        Some("Tv-series · Binnenkort")
    }
    fn choose_folder(&self) -> Option<&'static str> {
        Some("Kies map…")
    }
    fn choose_a_folder_for_your_library(&self) -> Option<&'static str> {
        Some("Kies een map voor je bibliotheek")
    }
    fn appearance(&self) -> Option<&'static str> {
        Some("Weergave")
    }
    fn theme(&self) -> Option<&'static str> {
        Some("Thema")
    }
    fn language(&self) -> Option<&'static str> {
        Some("Taal")
    }
    fn language_system(&self) -> Option<&'static str> {
        Some("Systeem ({0})")
    }
    fn dark(&self) -> Option<&'static str> {
        Some("Donker")
    }
    fn light(&self) -> Option<&'static str> {
        Some("Licht")
    }
    fn system(&self) -> Option<&'static str> {
        Some("Systeem")
    }
    fn playback(&self) -> Option<&'static str> {
        Some("Afspelen")
    }
    fn loudness(&self) -> Option<&'static str> {
        Some("Volumeniveau")
    }
    fn levels_tracks_to_the_same_loudness(&self) -> Option<&'static str> {
        Some(
            "Brengt nummers op dezelfde luidheid. Bestanden met ReplayGain-tags gebruiken die; andere bestanden worden op de achtergrond gemeten terwijl ze spelen en houden tot dan hun eigen niveau.",
        )
    }
    fn album_2(&self) -> Option<&'static str> {
        Some("Album")
    }
    fn track(&self) -> Option<&'static str> {
        Some("Nummer")
    }
    fn off(&self) -> Option<&'static str> {
        Some("Uit")
    }
    fn crossfade(&self) -> Option<&'static str> {
        Some("Crossfade")
    }
    fn each_track_fades_into_the_next(&self) -> Option<&'static str> {
        Some("Elk nummer vloeit in zoveel seconden over in het volgende. Nul zet het uit.")
    }
    fn look_up_synced_lyrics_on_lrclib(&self) -> Option<&'static str> {
        Some("Zoek gesynchroniseerde songteksten op lrclib.net")
    }
    fn for_tracks_with_no_lrc_file(&self) -> Option<&'static str> {
        Some(
            "Voor nummers zonder .lrc-bestand ernaast. Stuurt de titel, artiest, het album en de duur van elk nummer naar lrclib.net. Gevonden songteksten worden in de bibliotheek bewaard. Een .lrc-bestand gaat altijd voor.",
        )
    }
    fn movie_details(&self) -> Option<&'static str> {
        Some("Filmdetails")
    }
    fn descriptions_and_cast_for_movies_come(&self) -> Option<&'static str> {
        Some(
            "Beschrijvingen en cast van films komen van TMDb. Voer je eigen TMDb-API-sleutel in om ze te gebruiken. De sleutel wordt bewaard in je instellingenbestand, is alleen leesbaar voor je gebruikersaccount en wordt alleen naar TMDb gestuurd.",
        )
    }
    fn tmdb_api_key(&self) -> Option<&'static str> {
        Some("TMDb-API-sleutel")
    }
    fn save_key(&self) -> Option<&'static str> {
        Some("Bewaar sleutel")
    }
    fn remove_key(&self) -> Option<&'static str> {
        Some("Verwijder sleutel")
    }
    fn a_key_is_saved(&self) -> Option<&'static str> {
        Some("Er is een sleutel bewaard.")
    }
    fn no_key_saved_movies_show_no(&self) -> Option<&'static str> {
        Some("Geen sleutel bewaard. Films tonen geen beschrijving of cast tot je er een toevoegt.")
    }
    fn this_product_uses_the_tmdb_api(&self) -> Option<&'static str> {
        Some(
            "Dit product gebruikt de TMDB-API, maar wordt niet door TMDB ondersteund of gecertificeerd.",
        )
    }
    // side.rs
    fn nothing_playing_2(&self) -> Option<&'static str> {
        Some("Er wordt niets afgespeeld.")
    }
    fn this_track_has_no_lyrics_in(&self) -> Option<&'static str> {
        Some("Dit nummer heeft geen songtekst in de tags en geen .lrc-bestand ernaast.")
    }
    fn looking_up_lyrics_on_lrclib_net(&self) -> Option<&'static str> {
        Some("Songtekst zoeken op lrclib.net…")
    }
    fn no_lyrics_were_found_on_lrclib(&self) -> Option<&'static str> {
        Some("Geen songtekst gevonden op lrclib.net voor dit nummer.")
    }
    fn try_again(&self) -> Option<&'static str> {
        Some("Probeer opnieuw")
    }
    fn title(&self) -> Option<&'static str> {
        Some("Titel")
    }
    fn artist(&self) -> Option<&'static str> {
        Some("Artiest")
    }
    fn duration(&self) -> Option<&'static str> {
        Some("Duur")
    }
    fn size(&self) -> Option<&'static str> {
        Some("Grootte")
    }
    fn bitrate(&self) -> Option<&'static str> {
        Some("Bitsnelheid")
    }
    fn format(&self) -> Option<&'static str> {
        Some("Formaat")
    }
    fn channels(&self) -> Option<&'static str> {
        Some("Kanalen")
    }
    fn pictures(&self) -> Option<&'static str> {
        Some("Afbeeldingen")
    }
    fn file(&self) -> Option<&'static str> {
        Some("Bestand")
    }
    fn reveal_in_finder(&self) -> Option<&'static str> {
        Some("Toon in Finder")
    }
    fn tags(&self) -> Option<&'static str> {
        Some("Tags")
    }
    fn revert(&self) -> Option<&'static str> {
        Some("Herstel")
    }
    fn put_back_the_tags_the_file(&self) -> Option<&'static str> {
        Some("Zet de tags terug die het bestand had voordat het voor het eerst werd bewerkt")
    }
    fn look_up_synced_lyrics_on_lrclib_2(&self) -> Option<&'static str> {
        Some(
            "Gesynchroniseerde songteksten zoeken op lrclib.net? De titel, artiest, het album en de duur van elk nummer dat je opent, worden daarheen gestuurd.",
        )
    }
    fn turn_on_lyrics_lookup(&self) -> Option<&'static str> {
        Some("Zet songteksten zoeken aan")
    }
    fn photos_credits(&self) -> Option<&'static str> {
        Some("Foto’s: {0}")
    }
    fn duration_hours_minutes(&self) -> Option<&'static str> {
        Some("{0} u {1} min")
    }
    fn duration_minutes(&self) -> Option<&'static str> {
        Some("{0} min")
    }
    fn resume_at(&self) -> Option<&'static str> {
        Some("Ga verder bij {0}")
    }
    fn could_not_fetch_details(&self) -> Option<&'static str> {
        Some("Kan details niet ophalen: {0}")
    }
    fn fps_average_variable(&self) -> Option<&'static str> {
        Some("Gemiddeld {0} fps, variabel")
    }
    #[cfg(not(target_os = "macos"))]
    fn video_needs_libmpv(&self) -> Option<&'static str> {
        Some("Video vereist libmpv: {0}")
    }
    fn no_audio_output_error(&self) -> Option<&'static str> {
        Some("Geen audio-uitvoer: {0}")
    }
    fn could_not_play_station(&self) -> Option<&'static str> {
        Some("Kan de zender niet afspelen: {0}")
    }
    fn could_not_connect_station_error(&self) -> Option<&'static str> {
        Some("Kan geen verbinding maken met de zender: {0}")
    }
    fn could_not_switch_output(&self) -> Option<&'static str> {
        Some("Kan niet van uitvoer wisselen: {0}")
    }
    fn could_not_play_video(&self) -> Option<&'static str> {
        Some("Kan de video niet afspelen: {0}")
    }
    fn playback_stopped_early(&self) -> Option<&'static str> {
        Some("Afspelen voortijdig gestopt: {0}")
    }
    fn could_not_play_item(&self) -> Option<&'static str> {
        Some("Kan {0} niet afspelen: {1}")
    }
    fn stations_in_directory(&self) -> Option<&'static str> {
        Some("{0} zenders in de lijst · gesynchroniseerd {1}")
    }
    fn tuned_in(&self) -> Option<&'static str> {
        Some("{0} afgestemd")
    }
    fn tuned_in_suffix(&self) -> Option<&'static str> {
        Some("  ·  Afgestemd")
    }
    fn bitrate_at_least(&self) -> Option<&'static str> {
        Some("Bitsnelheid: {0}+ kbps")
    }
    fn never(&self) -> Option<&'static str> {
        Some("nooit")
    }
    fn minutes_ago(&self) -> Option<&'static str> {
        Some("{0} minuten geleden")
    }
    fn hours_ago(&self) -> Option<&'static str> {
        Some("{0} uur geleden")
    }
    fn days_ago(&self) -> Option<&'static str> {
        Some("{0} dagen geleden")
    }
    fn no_results_for(&self) -> Option<&'static str> {
        Some("Geen resultaten voor ‘{0}’.")
    }
    fn couldnt_reach_lrclib(&self) -> Option<&'static str> {
        Some("Kan lrclib.net niet bereiken: {0}")
    }
    fn radio_sync_failed(&self) -> Option<&'static str> {
        Some("Synchronisatie van de zenderlijst mislukt: {0}")
    }
    fn could_not_be_scanned(&self) -> Option<&'static str> {
        Some("{0}: kan niet worden gescand ({1}).")
    }
    fn one_movie(&self) -> Option<&'static str> {
        Some("{0} film")
    }
    fn many_movies(&self) -> Option<&'static str> {
        Some("{0} films")
    }
    fn one_album(&self) -> Option<&'static str> {
        Some("{0} album")
    }
    fn many_albums(&self) -> Option<&'static str> {
        Some("{0} albums")
    }
    fn one_track(&self) -> Option<&'static str> {
        Some("{0} nummer")
    }
    fn many_tracks(&self) -> Option<&'static str> {
        Some("{0} nummers")
    }
    fn one_folder(&self) -> Option<&'static str> {
        Some("{0} map")
    }
    fn many_folders(&self) -> Option<&'static str> {
        Some("{0} mappen")
    }
    fn one_file(&self) -> Option<&'static str> {
        Some("{0} bestand")
    }
    fn many_files(&self) -> Option<&'static str> {
        Some("{0} bestanden")
    }
}
