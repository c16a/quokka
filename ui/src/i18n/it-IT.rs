//! Italian. A string not here falls back to American English.

use super::Translation;

pub struct ItIt;

impl Translation for ItIt {
    fn language_tags(&self) -> &'static [&'static str] {
        &["it-IT", "it"]
    }

    fn native_name(&self) -> &'static str {
        "Italiano"
    }

    // albums.rs
    fn name(&self) -> Option<&'static str> {
        Some("Nome")
    }
    fn year(&self) -> Option<&'static str> {
        Some("Anno")
    }
    fn date_added(&self) -> Option<&'static str> {
        Some("Data di aggiunta")
    }
    fn albums(&self) -> Option<&'static str> {
        Some("Album")
    }
    fn no_albums_match_these_filters(&self) -> Option<&'static str> {
        Some("Nessun album corrisponde a questi filtri.")
    }
    fn no_albums_yet_choose_your_library(&self) -> Option<&'static str> {
        Some("Ancora nessun album. Scegli la cartella della libreria in Impostazioni.")
    }
    fn all_genres(&self) -> Option<&'static str> {
        Some("Tutti i generi")
    }
    fn all_years(&self) -> Option<&'static str> {
        Some("Tutti gli anni")
    }
    fn all_albums(&self) -> Option<&'static str> {
        Some("Tutti gli album")
    }
    fn unknown_album(&self) -> Option<&'static str> {
        Some("Album sconosciuto")
    }
    fn play(&self) -> Option<&'static str> {
        Some("Riproduci")
    }
    fn shuffle(&self) -> Option<&'static str> {
        Some("Casuale")
    }
    fn remove_from_favourites(&self) -> Option<&'static str> {
        Some("Rimuovi dai preferiti")
    }
    fn add_to_favourites(&self) -> Option<&'static str> {
        Some("Aggiungi ai preferiti")
    }
    fn album(&self) -> Option<&'static str> {
        Some("ALBUM")
    }
    // artists.rs
    fn no_artists_yet_choose_your_library(&self) -> Option<&'static str> {
        Some("Ancora nessun artista. Scegli la cartella della libreria in Impostazioni.")
    }
    fn artists(&self) -> Option<&'static str> {
        Some("Artisti")
    }
    fn all_artists(&self) -> Option<&'static str> {
        Some("Tutti gli artisti")
    }
    // format.rs
    fn mono(&self) -> Option<&'static str> {
        Some("Mono")
    }
    fn stereo(&self) -> Option<&'static str> {
        Some("Stereo")
    }
    // lib.rs
    fn add_a_folder_to_get_started(&self) -> Option<&'static str> {
        Some("Aggiungi una cartella per iniziare.")
    }
    fn syncing_the_radio_station_list(&self) -> Option<&'static str> {
        Some("Sincronizzazione dell’elenco delle stazioni…")
    }
    fn radio_station_sync_stopped_unexpectedly(&self) -> Option<&'static str> {
        Some("La sincronizzazione delle stazioni si è interrotta inaspettatamente.")
    }
    fn the_scan_stopped_unexpectedly(&self) -> Option<&'static str> {
        Some("la scansione si è interrotta inaspettatamente")
    }
    fn video(&self) -> Option<&'static str> {
        Some("Video")
    }
    fn search(&self) -> Option<&'static str> {
        Some("Cerca")
    }
    fn recent(&self) -> Option<&'static str> {
        Some("Recenti")
    }
    fn playlists(&self) -> Option<&'static str> {
        Some("Playlist")
    }
    fn radio(&self) -> Option<&'static str> {
        Some("Radio")
    }
    fn movies(&self) -> Option<&'static str> {
        Some("Film")
    }
    fn settings(&self) -> Option<&'static str> {
        Some("Impostazioni")
    }
    fn scanning(&self) -> Option<&'static str> {
        Some("Scansione…")
    }
    fn saved_the_tmdb_key(&self) -> Option<&'static str> {
        Some("Chiave TMDb salvata.")
    }
    fn could_not_save_settings(&self) -> Option<&'static str> {
        Some("Impossibile salvare le impostazioni: {0}")
    }
    fn radio_station_count(&self) -> Option<&'static str> {
        Some("Radio: {0} stazioni.")
    }
    fn scanning_folder(&self) -> Option<&'static str> {
        Some("Scansione di {0}…")
    }
    fn scanning_folder_queued(&self) -> Option<&'static str> {
        Some("Scansione di {0} (altre {1} in coda)…")
    }
    fn scan_summary(&self) -> Option<&'static str> {
        Some("{0}: {1} aggiunti o modificati, {2} invariati, {3} rimossi.")
    }
    fn matched_on_tmdb(&self) -> Option<&'static str> {
        Some("{0} associati su TMDb.")
    }
    fn tmdb_lookup_stopped(&self) -> Option<&'static str> {
        Some("Ricerca su TMDb interrotta ({0}); verrà ritentata alla prossima scansione.")
    }
    fn could_not_find_album(&self) -> Option<&'static str> {
        Some("Impossibile trovare l’album {0}")
    }
    fn could_not_tune_in(&self) -> Option<&'static str> {
        Some("Impossibile sintonizzarsi: {0}")
    }
    fn could_not_tune_out(&self) -> Option<&'static str> {
        Some("Impossibile smettere di ascoltare: {0}")
    }
    fn could_not_add_to_playlist(&self) -> Option<&'static str> {
        Some("Impossibile aggiungere alla playlist: {0}")
    }
    fn already_in_library(&self) -> Option<&'static str> {
        Some("{0} è già nella libreria.")
    }
    fn removed_folder(&self) -> Option<&'static str> {
        Some("{0} rimossa ({1}).")
    }
    fn could_not_remove_folder(&self) -> Option<&'static str> {
        Some("Impossibile rimuovere {0}: {1}")
    }
    fn hide_sidebar(&self) -> Option<&'static str> {
        Some("Nascondi barra laterale")
    }
    fn show_sidebar(&self) -> Option<&'static str> {
        Some("Mostra barra laterale")
    }
    // movies.rs
    fn no_movies_yet_choose_your_library(&self) -> Option<&'static str> {
        Some("Ancora nessun film. Scegli la cartella della libreria in Impostazioni.")
    }
    fn movie(&self) -> Option<&'static str> {
        Some("FILM")
    }
    fn length_unknown(&self) -> Option<&'static str> {
        Some("Durata sconosciuta")
    }
    fn start_over(&self) -> Option<&'static str> {
        Some("Ricomincia")
    }
    fn wrong_movie_choose_another(&self) -> Option<&'static str> {
        Some("Film sbagliato? Scegline un altro")
    }
    fn remove_details(&self) -> Option<&'static str> {
        Some("Rimuovi dettagli")
    }
    fn file_information(&self) -> Option<&'static str> {
        Some("Informazioni sul file")
    }
    fn source(&self) -> Option<&'static str> {
        Some("ORIGINE")
    }
    fn finding_details(&self) -> Option<&'static str> {
        Some("Ricerca dei dettagli…")
    }
    fn find_details_on_tmdb(&self) -> Option<&'static str> {
        Some("Cerca i dettagli su TMDb")
    }
    fn add_your_tmdb_api_key_in(&self) -> Option<&'static str> {
        Some("Aggiungi la tua chiave API di TMDb in Impostazioni per vedere trama e cast.")
    }
    fn synopsis(&self) -> Option<&'static str> {
        Some("Trama")
    }
    fn cast(&self) -> Option<&'static str> {
        Some("Cast")
    }
    fn director(&self) -> Option<&'static str> {
        Some("Regia")
    }
    fn writer(&self) -> Option<&'static str> {
        Some("Sceneggiatura")
    }
    fn crew(&self) -> Option<&'static str> {
        Some("Troupe")
    }
    fn which_movie_is_this(&self) -> Option<&'static str> {
        Some("Che film è questo?")
    }
    fn pick_the_one_your_file_is(&self) -> Option<&'static str> {
        Some(
            "Scegli quello che corrisponde al tuo file. Non viene salvato nulla finché non scegli.",
        )
    }
    fn none_of_these(&self) -> Option<&'static str> {
        Some("Nessuno di questi")
    }
    fn left_unmatched(&self) -> Option<&'static str> {
        Some("Lasciato senza corrispondenza.")
    }
    fn this_one(&self) -> Option<&'static str> {
        Some("Questo")
    }
    fn the_lookup_stopped_unexpectedly(&self) -> Option<&'static str> {
        Some("la ricerca si è interrotta inaspettatamente")
    }
    fn no_tmdb_details_for_this_movie(&self) -> Option<&'static str> {
        Some("Nessun dettaglio TMDb per questo film.")
    }
    fn unknown(&self) -> Option<&'static str> {
        Some("Sconosciuto")
    }
    fn container(&self) -> Option<&'static str> {
        Some("Contenitore")
    }
    fn codec(&self) -> Option<&'static str> {
        Some("Codec")
    }
    fn colour_depth(&self) -> Option<&'static str> {
        Some("Profondità colore")
    }
    fn chroma_subsampling(&self) -> Option<&'static str> {
        Some("Sottocampionamento cromatico")
    }
    fn dynamic_range(&self) -> Option<&'static str> {
        Some("Gamma dinamica")
    }
    fn resolution(&self) -> Option<&'static str> {
        Some("Risoluzione")
    }
    fn aspect_ratio(&self) -> Option<&'static str> {
        Some("Proporzioni")
    }
    fn frame_rate(&self) -> Option<&'static str> {
        Some("Frequenza fotogrammi")
    }
    fn length(&self) -> Option<&'static str> {
        Some("Durata")
    }
    fn file_size(&self) -> Option<&'static str> {
        Some("Dimensione file")
    }
    fn mpeg_transport_stream(&self) -> Option<&'static str> {
        Some("Flusso di trasporto MPEG")
    }
    fn mpeg_program_stream(&self) -> Option<&'static str> {
        Some("Flusso di programma MPEG")
    }
    fn raw_av1_stream_obu(&self) -> Option<&'static str> {
        Some("Flusso AV1 grezzo (OBU)")
    }
    // playback.rs
    fn no_audio_output_device_available(&self) -> Option<&'static str> {
        Some("nessun dispositivo di uscita audio disponibile")
    }
    fn could_not_connect_to_the_station(&self) -> Option<&'static str> {
        Some("Impossibile connettersi alla stazione.")
    }
    fn the_station_keeps_dropping_the_connection(&self) -> Option<&'static str> {
        Some("La stazione continua a interrompere la connessione. Premi riproduci per riprovare.")
    }
    fn pause(&self) -> Option<&'static str> {
        Some("Pausa")
    }
    fn connecting(&self) -> Option<&'static str> {
        Some("Connessione…")
    }
    fn paused(&self) -> Option<&'static str> {
        Some("In pausa")
    }
    fn live(&self) -> Option<&'static str> {
        Some("In diretta")
    }
    fn no_audio_output_device_available_2(&self) -> Option<&'static str> {
        Some("Nessun dispositivo di uscita audio disponibile")
    }
    fn back_10_seconds_left_arrow(&self) -> Option<&'static str> {
        Some("Indietro di 10 secondi (freccia sinistra)")
    }
    fn pause_space(&self) -> Option<&'static str> {
        Some("Pausa (Spazio)")
    }
    fn play_space(&self) -> Option<&'static str> {
        Some("Riproduci (Spazio)")
    }
    fn forward_10_seconds_right_arrow(&self) -> Option<&'static str> {
        Some("Avanti di 10 secondi (freccia destra)")
    }
    fn exit_full_screen_escape(&self) -> Option<&'static str> {
        Some("Esci da schermo intero (Esc)")
    }
    fn full_screen_f(&self) -> Option<&'static str> {
        Some("Schermo intero (F)")
    }
    fn unmute(&self) -> Option<&'static str> {
        Some("Riattiva audio")
    }
    fn mute(&self) -> Option<&'static str> {
        Some("Disattiva audio")
    }
    fn loading_video(&self) -> Option<&'static str> {
        Some("Caricamento del video…")
    }
    fn previous_track(&self) -> Option<&'static str> {
        Some("Brano precedente")
    }
    fn next_track(&self) -> Option<&'static str> {
        Some("Brano successivo")
    }
    fn repeat(&self) -> Option<&'static str> {
        Some("Ripeti")
    }
    fn nothing_playing(&self) -> Option<&'static str> {
        Some("Nessuna riproduzione")
    }
    fn output(&self) -> Option<&'static str> {
        Some("Uscita")
    }
    fn system_default(&self) -> Option<&'static str> {
        Some("Predefinita di sistema")
    }
    fn up_next(&self) -> Option<&'static str> {
        Some("In coda")
    }
    fn lyrics(&self) -> Option<&'static str> {
        Some("Testo")
    }
    fn playback_speed_click_to_change_speeds(&self) -> Option<&'static str> {
        Some(
            "Velocità di riproduzione. Fai clic per cambiarla. Le velocità diverse da 1× cambiano anche l’intonazione.",
        )
    }
    fn track_info(&self) -> Option<&'static str> {
        Some("Info brano")
    }
    // playlists.rs
    fn new_playlist_name(&self) -> Option<&'static str> {
        Some("Nome della nuova playlist")
    }
    fn create(&self) -> Option<&'static str> {
        Some("Crea")
    }
    fn no_playlists_yet_add_tracks_from(&self) -> Option<&'static str> {
        Some("Ancora nessuna playlist. Aggiungi brani da un album o dai risultati di ricerca.")
    }
    fn rename(&self) -> Option<&'static str> {
        Some("Rinomina")
    }
    fn delete_playlist(&self) -> Option<&'static str> {
        Some("Elimina playlist")
    }
    fn all_playlists(&self) -> Option<&'static str> {
        Some("Tutte le playlist")
    }
    fn save(&self) -> Option<&'static str> {
        Some("Salva")
    }
    fn cancel(&self) -> Option<&'static str> {
        Some("Annulla")
    }
    fn play_playlist(&self) -> Option<&'static str> {
        Some("Riproduci playlist")
    }
    fn shuffle_playlist(&self) -> Option<&'static str> {
        Some("Riproduci playlist in ordine casuale")
    }
    // queue.rs
    fn up_next_2(&self) -> Option<&'static str> {
        Some("In coda")
    }
    fn nothing_queued(&self) -> Option<&'static str> {
        Some("Nessun brano in coda")
    }
    fn clear(&self) -> Option<&'static str> {
        Some("Svuota")
    }
    fn tracks_you_queue_will_appear_here(&self) -> Option<&'static str> {
        Some("I brani che metti in coda appariranno qui.")
    }
    fn play_next(&self) -> Option<&'static str> {
        Some("Riproduci dopo")
    }
    fn add_to_queue(&self) -> Option<&'static str> {
        Some("Aggiungi alla coda")
    }
    fn add_to_playlist(&self) -> Option<&'static str> {
        Some("Aggiungi a playlist")
    }
    fn no_playlists_yet(&self) -> Option<&'static str> {
        Some("Ancora nessuna playlist")
    }
    // radio.rs
    fn no_stations_yet(&self) -> Option<&'static str> {
        Some("Ancora nessuna stazione.")
    }
    fn sync_now(&self) -> Option<&'static str> {
        Some("Sincronizza ora")
    }
    fn syncing(&self) -> Option<&'static str> {
        Some("Sincronizzazione…")
    }
    fn your_stations(&self) -> Option<&'static str> {
        Some("Le tue stazioni")
    }
    fn find_a_station_below_and_tune(&self) -> Option<&'static str> {
        Some("Trova una stazione qui sotto e sintonizzati. Apparirà qui.")
    }
    fn find_stations(&self) -> Option<&'static str> {
        Some("Trova stazioni")
    }
    fn sync_the_station_list_to_search(&self) -> Option<&'static str> {
        Some("Sincronizza l’elenco delle stazioni per cercarvi.")
    }
    fn no_stations_match(&self) -> Option<&'static str> {
        Some("Nessuna stazione corrisponde.")
    }
    fn country_any(&self) -> Option<&'static str> {
        Some("Paese: tutti")
    }
    fn any_country(&self) -> Option<&'static str> {
        Some("Tutti i paesi")
    }
    fn codec_any(&self) -> Option<&'static str> {
        Some("Codec: tutti")
    }
    fn any_codec(&self) -> Option<&'static str> {
        Some("Tutti i codec")
    }
    fn bitrate_any(&self) -> Option<&'static str> {
        Some("Bitrate: tutti")
    }
    fn any_bitrate(&self) -> Option<&'static str> {
        Some("Tutti i bitrate")
    }
    fn most_popular_first(&self) -> Option<&'static str> {
        Some("Prima le più popolari")
    }
    fn paused_2(&self) -> Option<&'static str> {
        Some("IN PAUSA")
    }
    fn live_now(&self) -> Option<&'static str> {
        Some("IN DIRETTA")
    }
    fn tune_out(&self) -> Option<&'static str> {
        Some("Smetti di ascoltare")
    }
    fn search_by_name_country_language_or(&self) -> Option<&'static str> {
        Some("Cerca per nome, paese, lingua o genere")
    }
    fn internet_radio(&self) -> Option<&'static str> {
        Some("Radio Internet")
    }
    fn just_now(&self) -> Option<&'static str> {
        Some("proprio ora")
    }
    // recent.rs
    fn recently_added(&self) -> Option<&'static str> {
        Some("Aggiunti di recente")
    }
    fn albums_you_add_to_the_library(&self) -> Option<&'static str> {
        Some("Gli album che aggiungi alla libreria appaiono qui.")
    }
    fn recently_played(&self) -> Option<&'static str> {
        Some("Ascoltati di recente")
    }
    fn albums_you_play_appear_here(&self) -> Option<&'static str> {
        Some("Gli album che ascolti appaiono qui.")
    }
    // search.rs
    fn search_albums_songs_playlists_and_stations(&self) -> Option<&'static str> {
        Some("Cerca album, brani, playlist e stazioni")
    }
    fn search_for_an_album_a_song(&self) -> Option<&'static str> {
        Some(
            "Cerca un album, un brano, una playlist o una stazione. Filtri: artist:, album:, genre:, year:, format:flac.",
        )
    }
    fn stations(&self) -> Option<&'static str> {
        Some("Stazioni")
    }
    fn songs(&self) -> Option<&'static str> {
        Some("Brani")
    }
    // settings_view.rs
    fn music(&self) -> Option<&'static str> {
        Some("Musica")
    }
    fn remove(&self) -> Option<&'static str> {
        Some("Rimuovi")
    }
    fn rescan(&self) -> Option<&'static str> {
        Some("Riesamina")
    }
    fn holds(&self) -> Option<&'static str> {
        Some("Contiene:")
    }
    fn library(&self) -> Option<&'static str> {
        Some("Libreria")
    }
    fn no_folders_yet(&self) -> Option<&'static str> {
        Some("Ancora nessuna cartella.")
    }
    fn rescan_all(&self) -> Option<&'static str> {
        Some("Riesamina tutto")
    }
    fn processing(&self) -> Option<&'static str> {
        Some("Elaborazione")
    }
    fn kind_not_set(&self) -> Option<&'static str> {
        Some("Tipo non impostato")
    }
    fn choose_what_each_of_these_folders(&self) -> Option<&'static str> {
        Some("Scegli cosa contiene ciascuna di queste cartelle.")
    }
    fn tv_shows(&self) -> Option<&'static str> {
        Some("Serie TV")
    }
    fn coming_soon(&self) -> Option<&'static str> {
        Some("In arrivo")
    }
    fn add_a_folder(&self) -> Option<&'static str> {
        Some("Aggiungi una cartella")
    }
    fn each_folder_holds_one_kind_of(&self) -> Option<&'static str> {
        Some(
            "Ogni cartella contiene un solo tipo di contenuto. Gli altri tipi di file al suo interno vengono ignorati.",
        )
    }
    fn tv_shows_soon(&self) -> Option<&'static str> {
        Some("Serie TV · In arrivo")
    }
    fn choose_folder(&self) -> Option<&'static str> {
        Some("Scegli cartella…")
    }
    fn choose_a_folder_for_your_library(&self) -> Option<&'static str> {
        Some("Scegli una cartella per la libreria")
    }
    fn appearance(&self) -> Option<&'static str> {
        Some("Aspetto")
    }
    fn theme(&self) -> Option<&'static str> {
        Some("Tema")
    }
    fn language(&self) -> Option<&'static str> {
        Some("Lingua")
    }
    fn language_system(&self) -> Option<&'static str> {
        Some("Sistema ({0})")
    }
    fn dark(&self) -> Option<&'static str> {
        Some("Scuro")
    }
    fn light(&self) -> Option<&'static str> {
        Some("Chiaro")
    }
    fn system(&self) -> Option<&'static str> {
        Some("Sistema")
    }
    fn playback(&self) -> Option<&'static str> {
        Some("Riproduzione")
    }
    fn loudness(&self) -> Option<&'static str> {
        Some("Volume uniforme")
    }
    fn levels_tracks_to_the_same_loudness(&self) -> Option<&'static str> {
        Some(
            "Porta i brani allo stesso volume percepito. I file con tag ReplayGain li usano; gli altri vengono misurati in background durante l’ascolto e mantengono il proprio livello fino ad allora.",
        )
    }
    fn album_2(&self) -> Option<&'static str> {
        Some("Album")
    }
    fn track(&self) -> Option<&'static str> {
        Some("Brano")
    }
    fn off(&self) -> Option<&'static str> {
        Some("No")
    }
    fn crossfade(&self) -> Option<&'static str> {
        Some("Dissolvenza incrociata")
    }
    fn each_track_fades_into_the_next(&self) -> Option<&'static str> {
        Some("Ogni brano sfuma nel successivo per questo numero di secondi. Zero la disattiva.")
    }
    fn look_up_synced_lyrics_on_lrclib(&self) -> Option<&'static str> {
        Some("Cerca testi sincronizzati su lrclib.net")
    }
    fn for_tracks_with_no_lrc_file(&self) -> Option<&'static str> {
        Some(
            "Per i brani senza un file .lrc accanto. Invia titolo, artista, album e durata di ogni brano a lrclib.net. I testi trovati vengono memorizzati nella libreria. Un file .lrc ha sempre la precedenza.",
        )
    }
    fn movie_details(&self) -> Option<&'static str> {
        Some("Dettagli dei film")
    }
    fn descriptions_and_cast_for_movies_come(&self) -> Option<&'static str> {
        Some(
            "Trame e cast dei film provengono da TMDb. Inserisci la tua chiave API di TMDb per usarli. La chiave è salvata nel file delle impostazioni, leggibile solo dal tuo account utente, e viene inviata solo a TMDb.",
        )
    }
    fn tmdb_api_key(&self) -> Option<&'static str> {
        Some("Chiave API di TMDb")
    }
    fn save_key(&self) -> Option<&'static str> {
        Some("Salva chiave")
    }
    fn remove_key(&self) -> Option<&'static str> {
        Some("Rimuovi chiave")
    }
    fn a_key_is_saved(&self) -> Option<&'static str> {
        Some("È salvata una chiave.")
    }
    fn no_key_saved_movies_show_no(&self) -> Option<&'static str> {
        Some(
            "Nessuna chiave salvata. I film non mostrano trame né cast finché non ne aggiungi una.",
        )
    }
    fn this_product_uses_the_tmdb_api(&self) -> Option<&'static str> {
        Some("Questo prodotto usa l’API di TMDB ma non è approvato né certificato da TMDB.")
    }
    // side.rs
    fn nothing_playing_2(&self) -> Option<&'static str> {
        Some("Nessuna riproduzione.")
    }
    fn this_track_has_no_lyrics_in(&self) -> Option<&'static str> {
        Some("Questo brano non ha un testo nei suoi tag né un file .lrc accanto.")
    }
    fn looking_up_lyrics_on_lrclib_net(&self) -> Option<&'static str> {
        Some("Ricerca del testo su lrclib.net…")
    }
    fn no_lyrics_were_found_on_lrclib(&self) -> Option<&'static str> {
        Some("Nessun testo trovato su lrclib.net per questo brano.")
    }
    fn try_again(&self) -> Option<&'static str> {
        Some("Riprova")
    }
    fn title(&self) -> Option<&'static str> {
        Some("Titolo")
    }
    fn artist(&self) -> Option<&'static str> {
        Some("Artista")
    }
    fn duration(&self) -> Option<&'static str> {
        Some("Durata")
    }
    fn size(&self) -> Option<&'static str> {
        Some("Dimensione")
    }
    fn bitrate(&self) -> Option<&'static str> {
        Some("Bitrate")
    }
    fn format(&self) -> Option<&'static str> {
        Some("Formato")
    }
    fn channels(&self) -> Option<&'static str> {
        Some("Canali")
    }
    fn pictures(&self) -> Option<&'static str> {
        Some("Immagini")
    }
    fn file(&self) -> Option<&'static str> {
        Some("File")
    }
    fn reveal_in_finder(&self) -> Option<&'static str> {
        Some("Mostra nel Finder")
    }
    fn tags(&self) -> Option<&'static str> {
        Some("Tag")
    }
    fn revert(&self) -> Option<&'static str> {
        Some("Ripristina")
    }
    fn put_back_the_tags_the_file(&self) -> Option<&'static str> {
        Some("Ripristina i tag che il file aveva prima della prima modifica")
    }
    fn look_up_synced_lyrics_on_lrclib_2(&self) -> Option<&'static str> {
        Some(
            "Cercare testi sincronizzati su lrclib.net? Titolo, artista, album e durata di ogni brano che apri vengono inviati lì.",
        )
    }
    fn turn_on_lyrics_lookup(&self) -> Option<&'static str> {
        Some("Attiva la ricerca dei testi")
    }
    fn photos_credits(&self) -> Option<&'static str> {
        Some("Foto: {0}")
    }
    fn duration_hours_minutes(&self) -> Option<&'static str> {
        Some("{0} h {1} min")
    }
    fn duration_minutes(&self) -> Option<&'static str> {
        Some("{0} min")
    }
    fn resume_at(&self) -> Option<&'static str> {
        Some("Riprendi da {0}")
    }
    fn could_not_fetch_details(&self) -> Option<&'static str> {
        Some("Impossibile ottenere i dettagli: {0}")
    }
    fn fps_average_variable(&self) -> Option<&'static str> {
        Some("{0} fps in media, variabile")
    }
    #[cfg(not(target_os = "macos"))]
    fn video_needs_libmpv(&self) -> Option<&'static str> {
        Some("Il video richiede libmpv: {0}")
    }
    fn no_audio_output_error(&self) -> Option<&'static str> {
        Some("Nessuna uscita audio: {0}")
    }
    fn could_not_play_station(&self) -> Option<&'static str> {
        Some("Impossibile riprodurre la stazione: {0}")
    }
    fn could_not_connect_station_error(&self) -> Option<&'static str> {
        Some("Impossibile connettersi alla stazione: {0}")
    }
    fn could_not_switch_output(&self) -> Option<&'static str> {
        Some("Impossibile cambiare uscita: {0}")
    }
    fn could_not_play_video(&self) -> Option<&'static str> {
        Some("Impossibile riprodurre il video: {0}")
    }
    fn playback_stopped_early(&self) -> Option<&'static str> {
        Some("Riproduzione interrotta: {0}")
    }
    fn could_not_play_item(&self) -> Option<&'static str> {
        Some("Impossibile riprodurre {0}: {1}")
    }
    fn stations_in_directory(&self) -> Option<&'static str> {
        Some("{0} stazioni nell’elenco · sincronizzato {1}")
    }
    fn tuned_in(&self) -> Option<&'static str> {
        Some("{0} sintonizzate")
    }
    fn tuned_in_suffix(&self) -> Option<&'static str> {
        Some("  ·  Sintonizzata")
    }
    fn bitrate_at_least(&self) -> Option<&'static str> {
        Some("Bitrate: {0}+ kbps")
    }
    fn never(&self) -> Option<&'static str> {
        Some("mai")
    }
    fn minutes_ago(&self) -> Option<&'static str> {
        Some("{0} minuti fa")
    }
    fn hours_ago(&self) -> Option<&'static str> {
        Some("{0} ore fa")
    }
    fn days_ago(&self) -> Option<&'static str> {
        Some("{0} giorni fa")
    }
    fn no_results_for(&self) -> Option<&'static str> {
        Some("Nessun risultato per «{0}».")
    }
    fn couldnt_reach_lrclib(&self) -> Option<&'static str> {
        Some("Impossibile raggiungere lrclib.net: {0}")
    }
    fn radio_sync_failed(&self) -> Option<&'static str> {
        Some("Sincronizzazione delle stazioni non riuscita: {0}")
    }
    fn could_not_be_scanned(&self) -> Option<&'static str> {
        Some("{0}: impossibile esaminare ({1}).")
    }
    fn one_movie(&self) -> Option<&'static str> {
        Some("{0} film")
    }
    fn many_movies(&self) -> Option<&'static str> {
        Some("{0} film")
    }
    fn one_album(&self) -> Option<&'static str> {
        Some("{0} album")
    }
    fn many_albums(&self) -> Option<&'static str> {
        Some("{0} album")
    }
    fn one_track(&self) -> Option<&'static str> {
        Some("{0} brano")
    }
    fn many_tracks(&self) -> Option<&'static str> {
        Some("{0} brani")
    }
    fn one_folder(&self) -> Option<&'static str> {
        Some("{0} cartella")
    }
    fn many_folders(&self) -> Option<&'static str> {
        Some("{0} cartelle")
    }
    fn one_file(&self) -> Option<&'static str> {
        Some("{0} file")
    }
    fn many_files(&self) -> Option<&'static str> {
        Some("{0} file")
    }
}
