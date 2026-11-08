//! French. A string not here falls back to American English.

use super::Translation;

pub struct FrFr;

impl Translation for FrFr {
    fn language_tags(&self) -> &'static [&'static str] {
        &["fr-FR", "fr"]
    }

    fn native_name(&self) -> &'static str {
        "Français"
    }

    fn is_singular(&self, count: usize) -> bool {
        count <= 1
    }

    // albums.rs
    fn name(&self) -> Option<&'static str> {
        Some("Nom")
    }
    fn year(&self) -> Option<&'static str> {
        Some("Année")
    }
    fn date_added(&self) -> Option<&'static str> {
        Some("Date d’ajout")
    }
    fn albums(&self) -> Option<&'static str> {
        Some("Albums")
    }
    fn no_albums_match_these_filters(&self) -> Option<&'static str> {
        Some("Aucun album ne correspond à ces filtres.")
    }
    fn no_albums_yet_choose_your_library(&self) -> Option<&'static str> {
        Some(
            "Aucun album pour l’instant. Choisissez le dossier de votre bibliothèque dans Réglages.",
        )
    }
    fn all_genres(&self) -> Option<&'static str> {
        Some("Tous les genres")
    }
    fn all_years(&self) -> Option<&'static str> {
        Some("Toutes les années")
    }
    fn all_albums(&self) -> Option<&'static str> {
        Some("Tous les albums")
    }
    fn unknown_album(&self) -> Option<&'static str> {
        Some("Album inconnu")
    }
    fn play(&self) -> Option<&'static str> {
        Some("Lire")
    }
    fn shuffle(&self) -> Option<&'static str> {
        Some("Aléatoire")
    }
    fn remove_from_favourites(&self) -> Option<&'static str> {
        Some("Retirer des favoris")
    }
    fn add_to_favourites(&self) -> Option<&'static str> {
        Some("Ajouter aux favoris")
    }
    fn album(&self) -> Option<&'static str> {
        Some("ALBUM")
    }
    // artists.rs
    fn no_artists_yet_choose_your_library(&self) -> Option<&'static str> {
        Some(
            "Aucun artiste pour l’instant. Choisissez le dossier de votre bibliothèque dans Réglages.",
        )
    }
    fn artists(&self) -> Option<&'static str> {
        Some("Artistes")
    }
    fn all_artists(&self) -> Option<&'static str> {
        Some("Tous les artistes")
    }
    // format.rs
    fn unavailable(&self) -> Option<&'static str> {
        Some("Indisponible")
    }
    fn mono(&self) -> Option<&'static str> {
        Some("Mono")
    }
    fn stereo(&self) -> Option<&'static str> {
        Some("Stéréo")
    }
    // lib.rs
    fn add_a_folder_to_get_started(&self) -> Option<&'static str> {
        Some("Ajoutez un dossier pour commencer.")
    }
    fn syncing_the_radio_station_list(&self) -> Option<&'static str> {
        Some("Synchronisation de la liste des stations…")
    }
    fn radio_station_sync_stopped_unexpectedly(&self) -> Option<&'static str> {
        Some("La synchronisation des stations s’est arrêtée de façon inattendue.")
    }
    fn the_scan_stopped_unexpectedly(&self) -> Option<&'static str> {
        Some("l’analyse s’est arrêtée de façon inattendue")
    }
    fn video(&self) -> Option<&'static str> {
        Some("Vidéo")
    }
    fn search(&self) -> Option<&'static str> {
        Some("Recherche")
    }
    fn recent(&self) -> Option<&'static str> {
        Some("Récents")
    }
    fn playlists(&self) -> Option<&'static str> {
        Some("Playlists")
    }
    fn radio(&self) -> Option<&'static str> {
        Some("Radio")
    }
    fn movies(&self) -> Option<&'static str> {
        Some("Films")
    }
    fn settings(&self) -> Option<&'static str> {
        Some("Réglages")
    }
    fn scanning(&self) -> Option<&'static str> {
        Some("Analyse…")
    }
    fn saved_the_tmdb_key(&self) -> Option<&'static str> {
        Some("Clé TMDb enregistrée.")
    }
    fn could_not_save_settings(&self) -> Option<&'static str> {
        Some("Impossible d’enregistrer les réglages : {0}")
    }
    fn radio_station_count(&self) -> Option<&'static str> {
        Some("Radio : {0} stations.")
    }
    fn scanning_folder(&self) -> Option<&'static str> {
        Some("Analyse de {0}…")
    }
    fn scanning_folder_queued(&self) -> Option<&'static str> {
        Some("Analyse de {0} ({1} autres en attente)…")
    }
    fn scan_summary(&self) -> Option<&'static str> {
        Some("{0} : {1} ajoutés ou modifiés, {2} inchangés, {3} supprimés.")
    }
    fn matched_on_tmdb(&self) -> Option<&'static str> {
        Some("{0} associés sur TMDb.")
    }
    fn tmdb_lookup_stopped(&self) -> Option<&'static str> {
        Some("Recherche TMDb interrompue ({0}) ; elle sera relancée à la prochaine analyse.")
    }
    fn could_not_find_album(&self) -> Option<&'static str> {
        Some("Impossible de trouver l’album {0}")
    }
    fn could_not_tune_in(&self) -> Option<&'static str> {
        Some("Impossible d’ajouter la station : {0}")
    }
    fn could_not_tune_out(&self) -> Option<&'static str> {
        Some("Impossible de retirer la station : {0}")
    }
    fn could_not_add_to_playlist(&self) -> Option<&'static str> {
        Some("Impossible d’ajouter à la playlist : {0}")
    }
    fn already_in_library(&self) -> Option<&'static str> {
        Some("{0} est déjà dans la bibliothèque.")
    }
    fn removed_folder(&self) -> Option<&'static str> {
        Some("{0} retiré ({1}).")
    }
    fn could_not_remove_folder(&self) -> Option<&'static str> {
        Some("Impossible de retirer {0} : {1}")
    }
    fn hide_sidebar(&self) -> Option<&'static str> {
        Some("Masquer la barre latérale")
    }
    fn show_sidebar(&self) -> Option<&'static str> {
        Some("Afficher la barre latérale")
    }
    // movies.rs
    fn no_movies_yet_choose_your_library(&self) -> Option<&'static str> {
        Some(
            "Aucun film pour l’instant. Choisissez le dossier de votre bibliothèque dans Réglages.",
        )
    }
    fn movie(&self) -> Option<&'static str> {
        Some("FILM")
    }
    fn length_unknown(&self) -> Option<&'static str> {
        Some("Durée inconnue")
    }
    fn start_over(&self) -> Option<&'static str> {
        Some("Reprendre au début")
    }
    fn wrong_movie_choose_another(&self) -> Option<&'static str> {
        Some("Mauvais film ? En choisir un autre")
    }
    fn remove_details(&self) -> Option<&'static str> {
        Some("Supprimer les détails")
    }
    fn file_information(&self) -> Option<&'static str> {
        Some("Informations sur le fichier")
    }
    fn source(&self) -> Option<&'static str> {
        Some("SOURCE")
    }
    fn finding_details(&self) -> Option<&'static str> {
        Some("Recherche des détails…")
    }
    fn find_details_on_tmdb(&self) -> Option<&'static str> {
        Some("Chercher les détails sur TMDb")
    }
    fn add_your_tmdb_api_key_in(&self) -> Option<&'static str> {
        Some("Ajoutez votre clé d’API TMDb dans Réglages pour voir le synopsis et la distribution.")
    }
    fn synopsis(&self) -> Option<&'static str> {
        Some("Synopsis")
    }
    fn cast(&self) -> Option<&'static str> {
        Some("Distribution")
    }
    fn director(&self) -> Option<&'static str> {
        Some("Réalisation")
    }
    fn writer(&self) -> Option<&'static str> {
        Some("Scénario")
    }
    fn crew(&self) -> Option<&'static str> {
        Some("Équipe")
    }
    fn which_movie_is_this(&self) -> Option<&'static str> {
        Some("De quel film s’agit-il ?")
    }
    fn pick_the_one_your_file_is(&self) -> Option<&'static str> {
        Some(
            "Choisissez celui qui correspond à votre fichier. Rien n’est enregistré avant votre choix.",
        )
    }
    fn none_of_these(&self) -> Option<&'static str> {
        Some("Aucun de ceux-ci")
    }
    fn left_unmatched(&self) -> Option<&'static str> {
        Some("Laissé sans correspondance.")
    }
    fn this_one(&self) -> Option<&'static str> {
        Some("Celui-ci")
    }
    fn the_lookup_stopped_unexpectedly(&self) -> Option<&'static str> {
        Some("la recherche s’est arrêtée de façon inattendue")
    }
    fn no_tmdb_details_for_this_movie(&self) -> Option<&'static str> {
        Some("Aucun détail TMDb pour ce film.")
    }
    fn unknown(&self) -> Option<&'static str> {
        Some("Inconnu")
    }
    fn container(&self) -> Option<&'static str> {
        Some("Conteneur")
    }
    fn codec(&self) -> Option<&'static str> {
        Some("Codec")
    }
    fn colour_depth(&self) -> Option<&'static str> {
        Some("Profondeur de couleur")
    }
    fn chroma_subsampling(&self) -> Option<&'static str> {
        Some("Sous-échantillonnage de la chrominance")
    }
    fn dynamic_range(&self) -> Option<&'static str> {
        Some("Plage dynamique")
    }
    fn resolution(&self) -> Option<&'static str> {
        Some("Résolution")
    }
    fn aspect_ratio(&self) -> Option<&'static str> {
        Some("Format d’image")
    }
    fn frame_rate(&self) -> Option<&'static str> {
        Some("Fréquence d’images")
    }
    fn length(&self) -> Option<&'static str> {
        Some("Durée")
    }
    fn file_size(&self) -> Option<&'static str> {
        Some("Taille du fichier")
    }
    fn mpeg_transport_stream(&self) -> Option<&'static str> {
        Some("Flux de transport MPEG")
    }
    fn mpeg_program_stream(&self) -> Option<&'static str> {
        Some("Flux de programme MPEG")
    }
    fn raw_av1_stream_obu(&self) -> Option<&'static str> {
        Some("Flux AV1 brut (OBU)")
    }
    // playback.rs
    fn no_audio_output_device_available(&self) -> Option<&'static str> {
        Some("aucun périphérique de sortie audio disponible")
    }
    fn could_not_connect_to_the_station(&self) -> Option<&'static str> {
        Some("Impossible de se connecter à la station.")
    }
    fn the_station_keeps_dropping_the_connection(&self) -> Option<&'static str> {
        Some("La station coupe sans cesse la connexion. Appuyez sur Lire pour réessayer.")
    }
    fn pause(&self) -> Option<&'static str> {
        Some("Pause")
    }
    fn connecting(&self) -> Option<&'static str> {
        Some("Connexion…")
    }
    fn paused(&self) -> Option<&'static str> {
        Some("En pause")
    }
    fn live(&self) -> Option<&'static str> {
        Some("En direct")
    }
    fn no_audio_output_device_available_2(&self) -> Option<&'static str> {
        Some("Aucun périphérique de sortie audio disponible")
    }
    fn back_10_seconds_left_arrow(&self) -> Option<&'static str> {
        Some("Reculer de 10 secondes (flèche gauche)")
    }
    fn pause_space(&self) -> Option<&'static str> {
        Some("Pause (Espace)")
    }
    fn play_space(&self) -> Option<&'static str> {
        Some("Lire (Espace)")
    }
    fn forward_10_seconds_right_arrow(&self) -> Option<&'static str> {
        Some("Avancer de 10 secondes (flèche droite)")
    }
    fn exit_full_screen_escape(&self) -> Option<&'static str> {
        Some("Quitter le plein écran (Échap)")
    }
    fn full_screen_f(&self) -> Option<&'static str> {
        Some("Plein écran (F)")
    }
    fn unmute(&self) -> Option<&'static str> {
        Some("Réactiver le son")
    }
    fn mute(&self) -> Option<&'static str> {
        Some("Couper le son")
    }
    fn loading_video(&self) -> Option<&'static str> {
        Some("Chargement de la vidéo…")
    }
    fn previous_track(&self) -> Option<&'static str> {
        Some("Morceau précédent")
    }
    fn next_track(&self) -> Option<&'static str> {
        Some("Morceau suivant")
    }
    fn repeat(&self) -> Option<&'static str> {
        Some("Répéter")
    }
    fn nothing_playing(&self) -> Option<&'static str> {
        Some("Aucune lecture")
    }
    fn output(&self) -> Option<&'static str> {
        Some("Sortie")
    }
    fn system_default(&self) -> Option<&'static str> {
        Some("Par défaut du système")
    }
    fn up_next(&self) -> Option<&'static str> {
        Some("À suivre")
    }
    fn lyrics(&self) -> Option<&'static str> {
        Some("Paroles")
    }
    fn playback_speed_click_to_change_speeds(&self) -> Option<&'static str> {
        Some(
            "Vitesse de lecture. Cliquez pour la changer. Toute vitesse autre que 1× modifie aussi la hauteur du son.",
        )
    }
    fn track_info(&self) -> Option<&'static str> {
        Some("Infos sur le morceau")
    }
    // playlists.rs
    fn new_playlist_name(&self) -> Option<&'static str> {
        Some("Nom de la nouvelle playlist")
    }
    fn create(&self) -> Option<&'static str> {
        Some("Créer")
    }
    fn no_playlists_yet_add_tracks_from(&self) -> Option<&'static str> {
        Some(
            "Aucune playlist pour l’instant. Ajoutez des morceaux depuis un album ou les résultats de recherche.",
        )
    }
    fn rename(&self) -> Option<&'static str> {
        Some("Renommer")
    }
    fn delete_playlist(&self) -> Option<&'static str> {
        Some("Supprimer la playlist")
    }
    fn all_playlists(&self) -> Option<&'static str> {
        Some("Toutes les playlists")
    }
    fn save(&self) -> Option<&'static str> {
        Some("Enregistrer")
    }
    fn cancel(&self) -> Option<&'static str> {
        Some("Annuler")
    }
    fn play_playlist(&self) -> Option<&'static str> {
        Some("Lire la playlist")
    }
    fn shuffle_playlist(&self) -> Option<&'static str> {
        Some("Lire la playlist en aléatoire")
    }
    // queue.rs
    fn up_next_2(&self) -> Option<&'static str> {
        Some("À suivre")
    }
    fn nothing_queued(&self) -> Option<&'static str> {
        Some("File d’attente vide")
    }
    fn clear(&self) -> Option<&'static str> {
        Some("Effacer")
    }
    fn tracks_you_queue_will_appear_here(&self) -> Option<&'static str> {
        Some("Les morceaux que vous ajoutez à la file apparaîtront ici.")
    }
    fn play_next(&self) -> Option<&'static str> {
        Some("Lire ensuite")
    }
    fn add_to_queue(&self) -> Option<&'static str> {
        Some("Ajouter à la file d’attente")
    }
    fn add_to_playlist(&self) -> Option<&'static str> {
        Some("Ajouter à une playlist")
    }
    fn no_playlists_yet(&self) -> Option<&'static str> {
        Some("Aucune playlist pour l’instant")
    }
    // radio.rs
    fn no_stations_yet(&self) -> Option<&'static str> {
        Some("Aucune station pour l’instant.")
    }
    fn sync_now(&self) -> Option<&'static str> {
        Some("Synchroniser")
    }
    fn syncing(&self) -> Option<&'static str> {
        Some("Synchronisation…")
    }
    fn your_stations(&self) -> Option<&'static str> {
        Some("Vos stations")
    }
    fn find_a_station_below_and_tune(&self) -> Option<&'static str> {
        Some("Trouvez une station ci-dessous et écoutez-la. Elle apparaîtra ici.")
    }
    fn find_stations(&self) -> Option<&'static str> {
        Some("Trouver des stations")
    }
    fn sync_the_station_list_to_search(&self) -> Option<&'static str> {
        Some("Synchronisez la liste des stations pour y chercher.")
    }
    fn no_stations_match(&self) -> Option<&'static str> {
        Some("Aucune station ne correspond.")
    }
    fn country_any(&self) -> Option<&'static str> {
        Some("Pays : tous")
    }
    fn any_country(&self) -> Option<&'static str> {
        Some("Tous les pays")
    }
    fn codec_any(&self) -> Option<&'static str> {
        Some("Codec : tous")
    }
    fn any_codec(&self) -> Option<&'static str> {
        Some("Tous les codecs")
    }
    fn bitrate_any(&self) -> Option<&'static str> {
        Some("Débit : tous")
    }
    fn any_bitrate(&self) -> Option<&'static str> {
        Some("Tous les débits")
    }
    fn most_popular_first(&self) -> Option<&'static str> {
        Some("Les plus populaires d’abord")
    }
    fn paused_2(&self) -> Option<&'static str> {
        Some("EN PAUSE")
    }
    fn live_now(&self) -> Option<&'static str> {
        Some("EN DIRECT")
    }
    fn tune_out(&self) -> Option<&'static str> {
        Some("Arrêter l’écoute")
    }
    fn search_by_name_country_language_or(&self) -> Option<&'static str> {
        Some("Chercher par nom, pays, langue ou genre")
    }
    fn internet_radio(&self) -> Option<&'static str> {
        Some("Radio Internet")
    }
    fn just_now(&self) -> Option<&'static str> {
        Some("à l’instant")
    }
    // recent.rs
    fn recently_added(&self) -> Option<&'static str> {
        Some("Ajouts récents")
    }
    fn albums_you_add_to_the_library(&self) -> Option<&'static str> {
        Some("Les albums que vous ajoutez à la bibliothèque apparaissent ici.")
    }
    fn recently_played(&self) -> Option<&'static str> {
        Some("Écoutés récemment")
    }
    fn albums_you_play_appear_here(&self) -> Option<&'static str> {
        Some("Les albums que vous écoutez apparaissent ici.")
    }
    // search.rs
    fn search_albums_songs_playlists_and_stations(&self) -> Option<&'static str> {
        Some("Chercher des albums, morceaux, playlists et stations")
    }
    fn search_for_an_album_a_song(&self) -> Option<&'static str> {
        Some(
            "Cherchez un album, un morceau, une playlist ou une station. Filtres : artist:, album:, genre:, year:, format:flac.",
        )
    }
    fn stations(&self) -> Option<&'static str> {
        Some("Stations")
    }
    fn songs(&self) -> Option<&'static str> {
        Some("Morceaux")
    }
    // settings_view.rs
    fn music(&self) -> Option<&'static str> {
        Some("Musique")
    }
    fn remove(&self) -> Option<&'static str> {
        Some("Retirer")
    }
    fn rescan(&self) -> Option<&'static str> {
        Some("Réanalyser")
    }
    fn holds(&self) -> Option<&'static str> {
        Some("Contient :")
    }
    fn library(&self) -> Option<&'static str> {
        Some("Bibliothèque")
    }
    fn no_folders_yet(&self) -> Option<&'static str> {
        Some("Aucun dossier pour l’instant.")
    }
    fn rescan_all(&self) -> Option<&'static str> {
        Some("Tout réanalyser")
    }
    fn processing(&self) -> Option<&'static str> {
        Some("Traitement")
    }
    fn kind_not_set(&self) -> Option<&'static str> {
        Some("Type non défini")
    }
    fn choose_what_each_of_these_folders(&self) -> Option<&'static str> {
        Some("Choisissez ce que contient chacun de ces dossiers.")
    }
    fn tv_shows(&self) -> Option<&'static str> {
        Some("Séries TV")
    }
    fn coming_soon(&self) -> Option<&'static str> {
        Some("Bientôt disponible")
    }
    fn add_a_folder(&self) -> Option<&'static str> {
        Some("Ajouter un dossier")
    }
    fn each_folder_holds_one_kind_of(&self) -> Option<&'static str> {
        Some(
            "Chaque dossier contient un seul type de média. Les autres types de fichiers qu’il contient sont ignorés.",
        )
    }
    fn tv_shows_soon(&self) -> Option<&'static str> {
        Some("Séries TV · Bientôt")
    }
    fn choose_folder(&self) -> Option<&'static str> {
        Some("Choisir un dossier…")
    }
    fn choose_a_folder_for_your_library(&self) -> Option<&'static str> {
        Some("Choisissez un dossier pour votre bibliothèque")
    }
    fn appearance(&self) -> Option<&'static str> {
        Some("Apparence")
    }
    fn theme(&self) -> Option<&'static str> {
        Some("Thème")
    }
    fn language(&self) -> Option<&'static str> {
        Some("Langue")
    }
    fn language_system(&self) -> Option<&'static str> {
        Some("Système ({0})")
    }
    fn dark(&self) -> Option<&'static str> {
        Some("Sombre")
    }
    fn light(&self) -> Option<&'static str> {
        Some("Clair")
    }
    fn system(&self) -> Option<&'static str> {
        Some("Système")
    }
    fn playback(&self) -> Option<&'static str> {
        Some("Lecture")
    }
    fn loudness(&self) -> Option<&'static str> {
        Some("Volume sonore")
    }
    fn levels_tracks_to_the_same_loudness(&self) -> Option<&'static str> {
        Some(
            "Égalise le volume sonore des morceaux. Les fichiers dotés de balises ReplayGain les utilisent ; les autres sont mesurés en arrière-plan pendant la lecture et gardent leur propre niveau d’ici là.",
        )
    }
    fn album_2(&self) -> Option<&'static str> {
        Some("Album")
    }
    fn track(&self) -> Option<&'static str> {
        Some("Morceau")
    }
    fn off(&self) -> Option<&'static str> {
        Some("Désactivé")
    }
    fn crossfade(&self) -> Option<&'static str> {
        Some("Fondu enchaîné")
    }
    fn each_track_fades_into_the_next(&self) -> Option<&'static str> {
        Some(
            "Chaque morceau se fond dans le suivant pendant ce nombre de secondes. Zéro le désactive.",
        )
    }
    fn look_up_synced_lyrics_on_lrclib(&self) -> Option<&'static str> {
        Some("Chercher des paroles synchronisées sur lrclib.net")
    }
    fn for_tracks_with_no_lrc_file(&self) -> Option<&'static str> {
        Some(
            "Pour les morceaux sans fichier .lrc à côté. Envoie le titre, l’artiste, l’album et la durée de chaque morceau à lrclib.net. Les paroles trouvées sont mises en cache dans la bibliothèque. Un fichier .lrc est toujours prioritaire.",
        )
    }
    fn movie_details(&self) -> Option<&'static str> {
        Some("Détails des films")
    }
    fn descriptions_and_cast_for_movies_come(&self) -> Option<&'static str> {
        Some(
            "Les synopsis et distributions des films proviennent de TMDb. Saisissez votre propre clé d’API TMDb pour les utiliser. La clé est enregistrée dans votre fichier de réglages, lisible uniquement par votre compte utilisateur, et n’est envoyée qu’à TMDb.",
        )
    }
    fn tmdb_api_key(&self) -> Option<&'static str> {
        Some("Clé d’API TMDb")
    }
    fn save_key(&self) -> Option<&'static str> {
        Some("Enregistrer la clé")
    }
    fn remove_key(&self) -> Option<&'static str> {
        Some("Supprimer la clé")
    }
    fn a_key_is_saved(&self) -> Option<&'static str> {
        Some("Une clé est enregistrée.")
    }
    fn no_key_saved_movies_show_no(&self) -> Option<&'static str> {
        Some(
            "Aucune clé enregistrée. Les films n’affichent ni synopsis ni distribution tant que vous n’en ajoutez pas.",
        )
    }
    fn this_product_uses_the_tmdb_api(&self) -> Option<&'static str> {
        Some("Ce produit utilise l’API TMDB, mais n’est ni approuvé ni certifié par TMDB.")
    }
    // side.rs
    fn nothing_playing_2(&self) -> Option<&'static str> {
        Some("Aucune lecture.")
    }
    fn this_track_has_no_lyrics_in(&self) -> Option<&'static str> {
        Some("Ce morceau n’a pas de paroles dans ses balises ni de fichier .lrc à côté.")
    }
    fn looking_up_lyrics_on_lrclib_net(&self) -> Option<&'static str> {
        Some("Recherche des paroles sur lrclib.net…")
    }
    fn no_lyrics_were_found_on_lrclib(&self) -> Option<&'static str> {
        Some("Aucune parole trouvée sur lrclib.net pour ce morceau.")
    }
    fn try_again(&self) -> Option<&'static str> {
        Some("Réessayer")
    }
    fn title(&self) -> Option<&'static str> {
        Some("Titre")
    }
    fn artist(&self) -> Option<&'static str> {
        Some("Artiste")
    }
    fn duration(&self) -> Option<&'static str> {
        Some("Durée")
    }
    fn size(&self) -> Option<&'static str> {
        Some("Taille")
    }
    fn bitrate(&self) -> Option<&'static str> {
        Some("Débit")
    }
    fn format(&self) -> Option<&'static str> {
        Some("Format")
    }
    fn channels(&self) -> Option<&'static str> {
        Some("Canaux")
    }
    fn pictures(&self) -> Option<&'static str> {
        Some("Images")
    }
    fn file(&self) -> Option<&'static str> {
        Some("Fichier")
    }
    fn reveal_in_finder(&self) -> Option<&'static str> {
        Some("Afficher dans le Finder")
    }
    fn tags(&self) -> Option<&'static str> {
        Some("Balises")
    }
    fn revert(&self) -> Option<&'static str> {
        Some("Rétablir")
    }
    fn put_back_the_tags_the_file(&self) -> Option<&'static str> {
        Some("Remettre les balises que le fichier avait avant sa première modification")
    }
    fn look_up_synced_lyrics_on_lrclib_2(&self) -> Option<&'static str> {
        Some(
            "Chercher des paroles synchronisées sur lrclib.net ? Le titre, l’artiste, l’album et la durée de chaque morceau que vous ouvrez y sont envoyés.",
        )
    }
    fn turn_on_lyrics_lookup(&self) -> Option<&'static str> {
        Some("Activer la recherche de paroles")
    }
    fn photos_credits(&self) -> Option<&'static str> {
        Some("Photos : {0}")
    }
    fn duration_hours_minutes(&self) -> Option<&'static str> {
        Some("{0} h {1} min")
    }
    fn duration_minutes(&self) -> Option<&'static str> {
        Some("{0} min")
    }
    fn resume_at(&self) -> Option<&'static str> {
        Some("Reprendre à {0}")
    }
    fn could_not_fetch_details(&self) -> Option<&'static str> {
        Some("Impossible d’obtenir les détails : {0}")
    }
    fn fps_average_variable(&self) -> Option<&'static str> {
        Some("{0} i/s en moyenne, variable")
    }
    #[cfg(not(target_os = "macos"))]
    fn video_needs_libmpv(&self) -> Option<&'static str> {
        Some("La vidéo nécessite libmpv : {0}")
    }
    fn no_audio_output_error(&self) -> Option<&'static str> {
        Some("Aucune sortie audio : {0}")
    }
    fn could_not_play_station(&self) -> Option<&'static str> {
        Some("Impossible de lire la station : {0}")
    }
    fn could_not_connect_station_error(&self) -> Option<&'static str> {
        Some("Impossible de se connecter à la station : {0}")
    }
    fn could_not_switch_output(&self) -> Option<&'static str> {
        Some("Impossible de changer de sortie : {0}")
    }
    fn could_not_play_video(&self) -> Option<&'static str> {
        Some("Impossible de lire la vidéo : {0}")
    }
    fn playback_stopped_early(&self) -> Option<&'static str> {
        Some("Lecture interrompue : {0}")
    }
    fn could_not_play_item(&self) -> Option<&'static str> {
        Some("Impossible de lire {0} : {1}")
    }
    fn stations_in_directory(&self) -> Option<&'static str> {
        Some("{0} stations dans l’annuaire · synchronisé {1}")
    }
    fn tuned_in(&self) -> Option<&'static str> {
        Some("{0} à l’écoute")
    }
    fn tuned_in_suffix(&self) -> Option<&'static str> {
        Some("  ·  À l’écoute")
    }
    fn bitrate_at_least(&self) -> Option<&'static str> {
        Some("Débit : {0}+ kbit/s")
    }
    fn never(&self) -> Option<&'static str> {
        Some("jamais")
    }
    fn minutes_ago(&self) -> Option<&'static str> {
        Some("il y a {0} minutes")
    }
    fn hours_ago(&self) -> Option<&'static str> {
        Some("il y a {0} heures")
    }
    fn days_ago(&self) -> Option<&'static str> {
        Some("il y a {0} jours")
    }
    fn no_results_for(&self) -> Option<&'static str> {
        Some("Aucun résultat pour « {0} ».")
    }
    fn couldnt_reach_lrclib(&self) -> Option<&'static str> {
        Some("Impossible de joindre lrclib.net : {0}")
    }
    fn radio_sync_failed(&self) -> Option<&'static str> {
        Some("Échec de la synchronisation des stations : {0}")
    }
    fn could_not_be_scanned(&self) -> Option<&'static str> {
        Some("{0} : impossible à analyser ({1}).")
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
        Some("{0} morceau")
    }
    fn many_tracks(&self) -> Option<&'static str> {
        Some("{0} morceaux")
    }
    fn one_folder(&self) -> Option<&'static str> {
        Some("{0} dossier")
    }
    fn many_folders(&self) -> Option<&'static str> {
        Some("{0} dossiers")
    }
    fn one_file(&self) -> Option<&'static str> {
        Some("{0} fichier")
    }
    fn many_files(&self) -> Option<&'static str> {
        Some("{0} fichiers")
    }
    fn audio_track(&self) -> Option<&'static str> {
        Some("Piste audio")
    }
    fn track_label(&self) -> Option<&'static str> {
        Some("Piste {0}")
    }
    fn deinterlace(&self) -> Option<&'static str> {
        Some("Désentrelacer")
    }
    fn screenshot_saved(&self) -> Option<&'static str> {
        Some("Capture d'écran enregistrée dans {0}")
    }
}
