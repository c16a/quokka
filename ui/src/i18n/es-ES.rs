//! Spanish (Spain), also served for other Spanish regions. A string not here falls back to American English.

use super::Translation;

pub struct EsEs;

impl Translation for EsEs {
    fn language_tags(&self) -> &'static [&'static str] {
        &["es-ES", "es"]
    }

    fn native_name(&self) -> &'static str {
        "Español"
    }

    // albums.rs
    fn name(&self) -> Option<&'static str> {
        Some("Nombre")
    }
    fn year(&self) -> Option<&'static str> {
        Some("Año")
    }
    fn date_added(&self) -> Option<&'static str> {
        Some("Fecha de adición")
    }
    fn albums(&self) -> Option<&'static str> {
        Some("Álbumes")
    }
    fn no_albums_match_these_filters(&self) -> Option<&'static str> {
        Some("Ningún álbum coincide con estos filtros.")
    }
    fn no_albums_yet_choose_your_library(&self) -> Option<&'static str> {
        Some("Todavía no hay álbumes. Elige la carpeta de tu biblioteca en Ajustes.")
    }
    fn all_genres(&self) -> Option<&'static str> {
        Some("Todos los géneros")
    }
    fn all_years(&self) -> Option<&'static str> {
        Some("Todos los años")
    }
    fn all_albums(&self) -> Option<&'static str> {
        Some("Todos los álbumes")
    }
    fn unknown_album(&self) -> Option<&'static str> {
        Some("Álbum desconocido")
    }
    fn play(&self) -> Option<&'static str> {
        Some("Reproducir")
    }
    fn shuffle(&self) -> Option<&'static str> {
        Some("Aleatorio")
    }
    fn remove_from_favourites(&self) -> Option<&'static str> {
        Some("Quitar de favoritos")
    }
    fn add_to_favourites(&self) -> Option<&'static str> {
        Some("Añadir a favoritos")
    }
    fn album(&self) -> Option<&'static str> {
        Some("ÁLBUM")
    }
    // artists.rs
    fn no_artists_yet_choose_your_library(&self) -> Option<&'static str> {
        Some("Todavía no hay artistas. Elige la carpeta de tu biblioteca en Ajustes.")
    }
    fn artists(&self) -> Option<&'static str> {
        Some("Artistas")
    }
    fn all_artists(&self) -> Option<&'static str> {
        Some("Todos los artistas")
    }
    // format.rs
    fn mono(&self) -> Option<&'static str> {
        Some("Mono")
    }
    fn stereo(&self) -> Option<&'static str> {
        Some("Estéreo")
    }
    // lib.rs
    fn add_a_folder_to_get_started(&self) -> Option<&'static str> {
        Some("Añade una carpeta para empezar.")
    }
    fn syncing_the_radio_station_list(&self) -> Option<&'static str> {
        Some("Sincronizando la lista de emisoras…")
    }
    fn radio_station_sync_stopped_unexpectedly(&self) -> Option<&'static str> {
        Some("La sincronización de emisoras se detuvo inesperadamente.")
    }
    fn the_scan_stopped_unexpectedly(&self) -> Option<&'static str> {
        Some("el análisis se detuvo inesperadamente")
    }
    fn video(&self) -> Option<&'static str> {
        Some("Vídeo")
    }
    fn search(&self) -> Option<&'static str> {
        Some("Buscar")
    }
    fn recent(&self) -> Option<&'static str> {
        Some("Recientes")
    }
    fn playlists(&self) -> Option<&'static str> {
        Some("Listas")
    }
    fn radio(&self) -> Option<&'static str> {
        Some("Radio")
    }
    fn movies(&self) -> Option<&'static str> {
        Some("Películas")
    }
    fn settings(&self) -> Option<&'static str> {
        Some("Ajustes")
    }
    fn scanning(&self) -> Option<&'static str> {
        Some("Analizando…")
    }
    fn saved_the_tmdb_key(&self) -> Option<&'static str> {
        Some("Clave de TMDb guardada.")
    }
    fn could_not_save_settings(&self) -> Option<&'static str> {
        Some("No se pudieron guardar los ajustes: {0}")
    }
    fn radio_station_count(&self) -> Option<&'static str> {
        Some("Radio: {0} emisoras.")
    }
    fn scanning_folder(&self) -> Option<&'static str> {
        Some("Analizando {0}…")
    }
    fn scanning_folder_queued(&self) -> Option<&'static str> {
        Some("Analizando {0} ({1} más en cola)…")
    }
    fn scan_summary(&self) -> Option<&'static str> {
        Some("{0}: {1} añadidos o modificados, {2} sin cambios, {3} eliminados.")
    }
    fn matched_on_tmdb(&self) -> Option<&'static str> {
        Some("{0} asociados en TMDb.")
    }
    fn tmdb_lookup_stopped(&self) -> Option<&'static str> {
        Some("La búsqueda en TMDb se detuvo ({0}); se volverá a intentar en el próximo análisis.")
    }
    fn could_not_find_album(&self) -> Option<&'static str> {
        Some("No se encontró el álbum {0}")
    }
    fn could_not_tune_in(&self) -> Option<&'static str> {
        Some("No se pudo sintonizar: {0}")
    }
    fn could_not_tune_out(&self) -> Option<&'static str> {
        Some("No se pudo dejar de escuchar: {0}")
    }
    fn could_not_add_to_playlist(&self) -> Option<&'static str> {
        Some("No se pudo añadir a la lista: {0}")
    }
    fn already_in_library(&self) -> Option<&'static str> {
        Some("{0} ya está en la biblioteca.")
    }
    fn removed_folder(&self) -> Option<&'static str> {
        Some("Se quitó {0} ({1}).")
    }
    fn could_not_remove_folder(&self) -> Option<&'static str> {
        Some("No se pudo quitar {0}: {1}")
    }
    fn hide_sidebar(&self) -> Option<&'static str> {
        Some("Ocultar barra lateral")
    }
    fn show_sidebar(&self) -> Option<&'static str> {
        Some("Mostrar barra lateral")
    }
    // movies.rs
    fn no_movies_yet_choose_your_library(&self) -> Option<&'static str> {
        Some("Todavía no hay películas. Elige la carpeta de tu biblioteca en Ajustes.")
    }
    fn movie(&self) -> Option<&'static str> {
        Some("PELÍCULA")
    }
    fn length_unknown(&self) -> Option<&'static str> {
        Some("Duración desconocida")
    }
    fn start_over(&self) -> Option<&'static str> {
        Some("Volver a empezar")
    }
    fn wrong_movie_choose_another(&self) -> Option<&'static str> {
        Some("¿No es esta película? Elige otra")
    }
    fn remove_details(&self) -> Option<&'static str> {
        Some("Quitar detalles")
    }
    fn file_information(&self) -> Option<&'static str> {
        Some("Información del archivo")
    }
    fn source(&self) -> Option<&'static str> {
        Some("ORIGEN")
    }
    fn finding_details(&self) -> Option<&'static str> {
        Some("Buscando detalles…")
    }
    fn find_details_on_tmdb(&self) -> Option<&'static str> {
        Some("Buscar detalles en TMDb")
    }
    fn add_your_tmdb_api_key_in(&self) -> Option<&'static str> {
        Some("Añade tu clave de API de TMDb en Ajustes para ver la sinopsis y el reparto.")
    }
    fn synopsis(&self) -> Option<&'static str> {
        Some("Sinopsis")
    }
    fn cast(&self) -> Option<&'static str> {
        Some("Reparto")
    }
    fn director(&self) -> Option<&'static str> {
        Some("Dirección")
    }
    fn writer(&self) -> Option<&'static str> {
        Some("Guion")
    }
    fn crew(&self) -> Option<&'static str> {
        Some("Equipo")
    }
    fn which_movie_is_this(&self) -> Option<&'static str> {
        Some("¿Qué película es esta?")
    }
    fn pick_the_one_your_file_is(&self) -> Option<&'static str> {
        Some("Elige la que corresponde a tu archivo. No se guarda nada hasta que elijas.")
    }
    fn none_of_these(&self) -> Option<&'static str> {
        Some("Ninguna de estas")
    }
    fn left_unmatched(&self) -> Option<&'static str> {
        Some("Sin asociar.")
    }
    fn this_one(&self) -> Option<&'static str> {
        Some("Esta")
    }
    fn the_lookup_stopped_unexpectedly(&self) -> Option<&'static str> {
        Some("la búsqueda se detuvo inesperadamente")
    }
    fn no_tmdb_details_for_this_movie(&self) -> Option<&'static str> {
        Some("No hay detalles de TMDb para esta película.")
    }
    fn unknown(&self) -> Option<&'static str> {
        Some("Desconocido")
    }
    fn container(&self) -> Option<&'static str> {
        Some("Contenedor")
    }
    fn codec(&self) -> Option<&'static str> {
        Some("Códec")
    }
    fn colour_depth(&self) -> Option<&'static str> {
        Some("Profundidad de color")
    }
    fn chroma_subsampling(&self) -> Option<&'static str> {
        Some("Submuestreo de croma")
    }
    fn dynamic_range(&self) -> Option<&'static str> {
        Some("Rango dinámico")
    }
    fn resolution(&self) -> Option<&'static str> {
        Some("Resolución")
    }
    fn aspect_ratio(&self) -> Option<&'static str> {
        Some("Relación de aspecto")
    }
    fn frame_rate(&self) -> Option<&'static str> {
        Some("Fotogramas por segundo")
    }
    fn length(&self) -> Option<&'static str> {
        Some("Duración")
    }
    fn file_size(&self) -> Option<&'static str> {
        Some("Tamaño del archivo")
    }
    fn mpeg_transport_stream(&self) -> Option<&'static str> {
        Some("Flujo de transporte MPEG")
    }
    fn mpeg_program_stream(&self) -> Option<&'static str> {
        Some("Flujo de programa MPEG")
    }
    fn raw_av1_stream_obu(&self) -> Option<&'static str> {
        Some("Flujo AV1 sin procesar (OBU)")
    }
    // playback.rs
    fn no_audio_output_device_available(&self) -> Option<&'static str> {
        Some("no hay ningún dispositivo de salida de audio disponible")
    }
    fn could_not_connect_to_the_station(&self) -> Option<&'static str> {
        Some("No se pudo conectar con la emisora.")
    }
    fn the_station_keeps_dropping_the_connection(&self) -> Option<&'static str> {
        Some(
            "La emisora corta la conexión una y otra vez. Pulsa reproducir para volver a intentarlo.",
        )
    }
    fn pause(&self) -> Option<&'static str> {
        Some("Pausa")
    }
    fn connecting(&self) -> Option<&'static str> {
        Some("Conectando…")
    }
    fn paused(&self) -> Option<&'static str> {
        Some("En pausa")
    }
    fn live(&self) -> Option<&'static str> {
        Some("En directo")
    }
    fn no_audio_output_device_available_2(&self) -> Option<&'static str> {
        Some("No hay ningún dispositivo de salida de audio disponible")
    }
    fn back_10_seconds_left_arrow(&self) -> Option<&'static str> {
        Some("Retroceder 10 segundos (flecha izquierda)")
    }
    fn pause_space(&self) -> Option<&'static str> {
        Some("Pausa (Espacio)")
    }
    fn play_space(&self) -> Option<&'static str> {
        Some("Reproducir (Espacio)")
    }
    fn forward_10_seconds_right_arrow(&self) -> Option<&'static str> {
        Some("Avanzar 10 segundos (flecha derecha)")
    }
    fn exit_full_screen_escape(&self) -> Option<&'static str> {
        Some("Salir de pantalla completa (Esc)")
    }
    fn full_screen_f(&self) -> Option<&'static str> {
        Some("Pantalla completa (F)")
    }
    fn unmute(&self) -> Option<&'static str> {
        Some("Activar sonido")
    }
    fn mute(&self) -> Option<&'static str> {
        Some("Silenciar")
    }
    fn loading_video(&self) -> Option<&'static str> {
        Some("Cargando vídeo…")
    }
    fn previous_track(&self) -> Option<&'static str> {
        Some("Canción anterior")
    }
    fn next_track(&self) -> Option<&'static str> {
        Some("Canción siguiente")
    }
    fn repeat(&self) -> Option<&'static str> {
        Some("Repetir")
    }
    fn nothing_playing(&self) -> Option<&'static str> {
        Some("No se está reproduciendo nada")
    }
    fn output(&self) -> Option<&'static str> {
        Some("Salida")
    }
    fn system_default(&self) -> Option<&'static str> {
        Some("Predeterminada del sistema")
    }
    fn up_next(&self) -> Option<&'static str> {
        Some("A continuación")
    }
    fn lyrics(&self) -> Option<&'static str> {
        Some("Letra")
    }
    fn playback_speed_click_to_change_speeds(&self) -> Option<&'static str> {
        Some(
            "Velocidad de reproducción. Haz clic para cambiarla. Las velocidades distintas de 1× también cambian el tono.",
        )
    }
    fn track_info(&self) -> Option<&'static str> {
        Some("Información de la canción")
    }
    // playlists.rs
    fn new_playlist_name(&self) -> Option<&'static str> {
        Some("Nombre de la nueva lista")
    }
    fn create(&self) -> Option<&'static str> {
        Some("Crear")
    }
    fn no_playlists_yet_add_tracks_from(&self) -> Option<&'static str> {
        Some("Todavía no hay listas. Añade canciones desde un álbum o los resultados de búsqueda.")
    }
    fn rename(&self) -> Option<&'static str> {
        Some("Renombrar")
    }
    fn delete_playlist(&self) -> Option<&'static str> {
        Some("Eliminar lista")
    }
    fn all_playlists(&self) -> Option<&'static str> {
        Some("Todas las listas")
    }
    fn save(&self) -> Option<&'static str> {
        Some("Guardar")
    }
    fn cancel(&self) -> Option<&'static str> {
        Some("Cancelar")
    }
    fn play_playlist(&self) -> Option<&'static str> {
        Some("Reproducir lista")
    }
    fn shuffle_playlist(&self) -> Option<&'static str> {
        Some("Reproducir lista en aleatorio")
    }
    // queue.rs
    fn up_next_2(&self) -> Option<&'static str> {
        Some("A continuación")
    }
    fn nothing_queued(&self) -> Option<&'static str> {
        Some("No hay nada en la cola")
    }
    fn clear(&self) -> Option<&'static str> {
        Some("Vaciar")
    }
    fn tracks_you_queue_will_appear_here(&self) -> Option<&'static str> {
        Some("Las canciones que pongas en cola aparecerán aquí.")
    }
    fn play_next(&self) -> Option<&'static str> {
        Some("Reproducir a continuación")
    }
    fn add_to_queue(&self) -> Option<&'static str> {
        Some("Añadir a la cola")
    }
    fn add_to_playlist(&self) -> Option<&'static str> {
        Some("Añadir a una lista")
    }
    fn no_playlists_yet(&self) -> Option<&'static str> {
        Some("Todavía no hay listas")
    }
    // radio.rs
    fn no_stations_yet(&self) -> Option<&'static str> {
        Some("Todavía no hay emisoras.")
    }
    fn sync_now(&self) -> Option<&'static str> {
        Some("Sincronizar ahora")
    }
    fn syncing(&self) -> Option<&'static str> {
        Some("Sincronizando…")
    }
    fn your_stations(&self) -> Option<&'static str> {
        Some("Tus emisoras")
    }
    fn find_a_station_below_and_tune(&self) -> Option<&'static str> {
        Some("Busca una emisora abajo y sintonízala. Aparecerá aquí.")
    }
    fn find_stations(&self) -> Option<&'static str> {
        Some("Buscar emisoras")
    }
    fn sync_the_station_list_to_search(&self) -> Option<&'static str> {
        Some("Sincroniza la lista de emisoras para buscar en ella.")
    }
    fn no_stations_match(&self) -> Option<&'static str> {
        Some("Ninguna emisora coincide.")
    }
    fn country_any(&self) -> Option<&'static str> {
        Some("País: cualquiera")
    }
    fn any_country(&self) -> Option<&'static str> {
        Some("Cualquier país")
    }
    fn codec_any(&self) -> Option<&'static str> {
        Some("Códec: cualquiera")
    }
    fn any_codec(&self) -> Option<&'static str> {
        Some("Cualquier códec")
    }
    fn bitrate_any(&self) -> Option<&'static str> {
        Some("Tasa de bits: cualquiera")
    }
    fn any_bitrate(&self) -> Option<&'static str> {
        Some("Cualquier tasa de bits")
    }
    fn most_popular_first(&self) -> Option<&'static str> {
        Some("Más populares primero")
    }
    fn paused_2(&self) -> Option<&'static str> {
        Some("EN PAUSA")
    }
    fn live_now(&self) -> Option<&'static str> {
        Some("EN DIRECTO")
    }
    fn tune_out(&self) -> Option<&'static str> {
        Some("Dejar de escuchar")
    }
    fn search_by_name_country_language_or(&self) -> Option<&'static str> {
        Some("Buscar por nombre, país, idioma o género")
    }
    fn internet_radio(&self) -> Option<&'static str> {
        Some("Radio por Internet")
    }
    fn just_now(&self) -> Option<&'static str> {
        Some("ahora mismo")
    }
    // recent.rs
    fn recently_added(&self) -> Option<&'static str> {
        Some("Añadido recientemente")
    }
    fn albums_you_add_to_the_library(&self) -> Option<&'static str> {
        Some("Los álbumes que añadas a la biblioteca aparecen aquí.")
    }
    fn recently_played(&self) -> Option<&'static str> {
        Some("Reproducido recientemente")
    }
    fn albums_you_play_appear_here(&self) -> Option<&'static str> {
        Some("Los álbumes que reproduzcas aparecen aquí.")
    }
    // search.rs
    fn search_albums_songs_playlists_and_stations(&self) -> Option<&'static str> {
        Some("Buscar álbumes, canciones, listas y emisoras")
    }
    fn search_for_an_album_a_song(&self) -> Option<&'static str> {
        Some(
            "Busca un álbum, una canción, una lista o una emisora. Filtros: artist:, album:, genre:, year:, format:flac.",
        )
    }
    fn stations(&self) -> Option<&'static str> {
        Some("Emisoras")
    }
    fn songs(&self) -> Option<&'static str> {
        Some("Canciones")
    }
    // settings_view.rs
    fn music(&self) -> Option<&'static str> {
        Some("Música")
    }
    fn remove(&self) -> Option<&'static str> {
        Some("Quitar")
    }
    fn rescan(&self) -> Option<&'static str> {
        Some("Volver a analizar")
    }
    fn holds(&self) -> Option<&'static str> {
        Some("Contiene:")
    }
    fn library(&self) -> Option<&'static str> {
        Some("Biblioteca")
    }
    fn no_folders_yet(&self) -> Option<&'static str> {
        Some("Todavía no hay carpetas.")
    }
    fn rescan_all(&self) -> Option<&'static str> {
        Some("Volver a analizar todo")
    }
    fn processing(&self) -> Option<&'static str> {
        Some("Procesando")
    }
    fn kind_not_set(&self) -> Option<&'static str> {
        Some("Tipo sin definir")
    }
    fn choose_what_each_of_these_folders(&self) -> Option<&'static str> {
        Some("Elige qué contiene cada una de estas carpetas.")
    }
    fn tv_shows(&self) -> Option<&'static str> {
        Some("Series")
    }
    fn coming_soon(&self) -> Option<&'static str> {
        Some("Próximamente")
    }
    fn add_a_folder(&self) -> Option<&'static str> {
        Some("Añadir una carpeta")
    }
    fn each_folder_holds_one_kind_of(&self) -> Option<&'static str> {
        Some(
            "Cada carpeta contiene un solo tipo de contenido. Los demás tipos de archivo que haya en ella se omiten.",
        )
    }
    fn tv_shows_soon(&self) -> Option<&'static str> {
        Some("Series · Próximamente")
    }
    fn choose_folder(&self) -> Option<&'static str> {
        Some("Elegir carpeta…")
    }
    fn choose_a_folder_for_your_library(&self) -> Option<&'static str> {
        Some("Elige una carpeta para tu biblioteca")
    }
    fn appearance(&self) -> Option<&'static str> {
        Some("Apariencia")
    }
    fn theme(&self) -> Option<&'static str> {
        Some("Tema")
    }
    fn language(&self) -> Option<&'static str> {
        Some("Idioma")
    }
    fn language_system(&self) -> Option<&'static str> {
        Some("Sistema ({0})")
    }
    fn dark(&self) -> Option<&'static str> {
        Some("Oscuro")
    }
    fn light(&self) -> Option<&'static str> {
        Some("Claro")
    }
    fn system(&self) -> Option<&'static str> {
        Some("Sistema")
    }
    fn playback(&self) -> Option<&'static str> {
        Some("Reproducción")
    }
    fn loudness(&self) -> Option<&'static str> {
        Some("Volumen uniforme")
    }
    fn levels_tracks_to_the_same_loudness(&self) -> Option<&'static str> {
        Some(
            "Iguala el volumen de las canciones. Los archivos con etiquetas ReplayGain las usan; los demás se miden en segundo plano mientras suenan y conservan su propio nivel hasta entonces.",
        )
    }
    fn album_2(&self) -> Option<&'static str> {
        Some("Álbum")
    }
    fn track(&self) -> Option<&'static str> {
        Some("Canción")
    }
    fn off(&self) -> Option<&'static str> {
        Some("Desactivado")
    }
    fn crossfade(&self) -> Option<&'static str> {
        Some("Fundido")
    }
    fn each_track_fades_into_the_next(&self) -> Option<&'static str> {
        Some("Cada canción se funde con la siguiente durante estos segundos. Cero lo desactiva.")
    }
    fn look_up_synced_lyrics_on_lrclib(&self) -> Option<&'static str> {
        Some("Buscar letras sincronizadas en lrclib.net")
    }
    fn for_tracks_with_no_lrc_file(&self) -> Option<&'static str> {
        Some(
            "Para canciones sin un archivo .lrc al lado. Envía el título, el artista, el álbum y la duración de cada canción a lrclib.net. Las letras encontradas se guardan en la biblioteca. Un archivo .lrc siempre tiene prioridad.",
        )
    }
    fn movie_details(&self) -> Option<&'static str> {
        Some("Detalles de películas")
    }
    fn descriptions_and_cast_for_movies_come(&self) -> Option<&'static str> {
        Some(
            "Las sinopsis y los repartos de las películas proceden de TMDb. Introduce tu propia clave de API de TMDb para usarlos. La clave se guarda en tu archivo de ajustes, solo la puede leer tu cuenta de usuario y solo se envía a TMDb.",
        )
    }
    fn tmdb_api_key(&self) -> Option<&'static str> {
        Some("Clave de API de TMDb")
    }
    fn save_key(&self) -> Option<&'static str> {
        Some("Guardar clave")
    }
    fn remove_key(&self) -> Option<&'static str> {
        Some("Quitar clave")
    }
    fn a_key_is_saved(&self) -> Option<&'static str> {
        Some("Hay una clave guardada.")
    }
    fn no_key_saved_movies_show_no(&self) -> Option<&'static str> {
        Some(
            "No hay ninguna clave guardada. Las películas no muestran sinopsis ni reparto hasta que añadas una.",
        )
    }
    fn this_product_uses_the_tmdb_api(&self) -> Option<&'static str> {
        Some("Este producto usa la API de TMDB, pero TMDB no lo respalda ni lo certifica.")
    }
    // side.rs
    fn nothing_playing_2(&self) -> Option<&'static str> {
        Some("No se está reproduciendo nada.")
    }
    fn this_track_has_no_lyrics_in(&self) -> Option<&'static str> {
        Some("Esta canción no tiene letra en sus etiquetas ni un archivo .lrc al lado.")
    }
    fn looking_up_lyrics_on_lrclib_net(&self) -> Option<&'static str> {
        Some("Buscando la letra en lrclib.net…")
    }
    fn no_lyrics_were_found_on_lrclib(&self) -> Option<&'static str> {
        Some("No se encontró la letra de esta canción en lrclib.net.")
    }
    fn try_again(&self) -> Option<&'static str> {
        Some("Reintentar")
    }
    fn title(&self) -> Option<&'static str> {
        Some("Título")
    }
    fn artist(&self) -> Option<&'static str> {
        Some("Artista")
    }
    fn duration(&self) -> Option<&'static str> {
        Some("Duración")
    }
    fn size(&self) -> Option<&'static str> {
        Some("Tamaño")
    }
    fn bitrate(&self) -> Option<&'static str> {
        Some("Tasa de bits")
    }
    fn format(&self) -> Option<&'static str> {
        Some("Formato")
    }
    fn channels(&self) -> Option<&'static str> {
        Some("Canales")
    }
    fn pictures(&self) -> Option<&'static str> {
        Some("Imágenes")
    }
    fn file(&self) -> Option<&'static str> {
        Some("Archivo")
    }
    fn reveal_in_finder(&self) -> Option<&'static str> {
        Some("Mostrar en el Finder")
    }
    fn tags(&self) -> Option<&'static str> {
        Some("Etiquetas")
    }
    fn revert(&self) -> Option<&'static str> {
        Some("Restaurar")
    }
    fn put_back_the_tags_the_file(&self) -> Option<&'static str> {
        Some("Restaurar las etiquetas que tenía el archivo antes de editarlo por primera vez")
    }
    fn look_up_synced_lyrics_on_lrclib_2(&self) -> Option<&'static str> {
        Some(
            "¿Buscar letras sincronizadas en lrclib.net? Se envían allí el título, el artista, el álbum y la duración de cada canción que abras.",
        )
    }
    fn turn_on_lyrics_lookup(&self) -> Option<&'static str> {
        Some("Activar la búsqueda de letras")
    }
    fn photos_credits(&self) -> Option<&'static str> {
        Some("Fotos: {0}")
    }
    fn duration_hours_minutes(&self) -> Option<&'static str> {
        Some("{0} h {1} min")
    }
    fn duration_minutes(&self) -> Option<&'static str> {
        Some("{0} min")
    }
    fn resume_at(&self) -> Option<&'static str> {
        Some("Reanudar en {0}")
    }
    fn could_not_fetch_details(&self) -> Option<&'static str> {
        Some("No se pudieron obtener los detalles: {0}")
    }
    fn fps_average_variable(&self) -> Option<&'static str> {
        Some("{0} fps de media, variable")
    }
    #[cfg(not(target_os = "macos"))]
    fn video_needs_libmpv(&self) -> Option<&'static str> {
        Some("El vídeo necesita libmpv: {0}")
    }
    fn no_audio_output_error(&self) -> Option<&'static str> {
        Some("Sin salida de audio: {0}")
    }
    fn could_not_play_station(&self) -> Option<&'static str> {
        Some("No se pudo reproducir la emisora: {0}")
    }
    fn could_not_connect_station_error(&self) -> Option<&'static str> {
        Some("No se pudo conectar con la emisora: {0}")
    }
    fn could_not_switch_output(&self) -> Option<&'static str> {
        Some("No se pudo cambiar la salida: {0}")
    }
    fn could_not_play_video(&self) -> Option<&'static str> {
        Some("No se pudo reproducir el vídeo: {0}")
    }
    fn playback_stopped_early(&self) -> Option<&'static str> {
        Some("La reproducción se detuvo antes de tiempo: {0}")
    }
    fn could_not_play_item(&self) -> Option<&'static str> {
        Some("No se pudo reproducir {0}: {1}")
    }
    fn stations_in_directory(&self) -> Option<&'static str> {
        Some("{0} emisoras en el directorio · sincronizado {1}")
    }
    fn tuned_in(&self) -> Option<&'static str> {
        Some("{0} sintonizadas")
    }
    fn tuned_in_suffix(&self) -> Option<&'static str> {
        Some("  ·  Sintonizada")
    }
    fn bitrate_at_least(&self) -> Option<&'static str> {
        Some("Tasa de bits: {0}+ kbps")
    }
    fn never(&self) -> Option<&'static str> {
        Some("nunca")
    }
    fn minutes_ago(&self) -> Option<&'static str> {
        Some("hace {0} minutos")
    }
    fn hours_ago(&self) -> Option<&'static str> {
        Some("hace {0} horas")
    }
    fn days_ago(&self) -> Option<&'static str> {
        Some("hace {0} días")
    }
    fn no_results_for(&self) -> Option<&'static str> {
        Some("No hay resultados para «{0}».")
    }
    fn couldnt_reach_lrclib(&self) -> Option<&'static str> {
        Some("No se pudo conectar con lrclib.net: {0}")
    }
    fn radio_sync_failed(&self) -> Option<&'static str> {
        Some("Falló la sincronización de emisoras: {0}")
    }
    fn could_not_be_scanned(&self) -> Option<&'static str> {
        Some("{0}: no se pudo analizar ({1}).")
    }
    fn one_movie(&self) -> Option<&'static str> {
        Some("{0} película")
    }
    fn many_movies(&self) -> Option<&'static str> {
        Some("{0} películas")
    }
    fn one_album(&self) -> Option<&'static str> {
        Some("{0} álbum")
    }
    fn many_albums(&self) -> Option<&'static str> {
        Some("{0} álbumes")
    }
    fn one_track(&self) -> Option<&'static str> {
        Some("{0} canción")
    }
    fn many_tracks(&self) -> Option<&'static str> {
        Some("{0} canciones")
    }
    fn one_folder(&self) -> Option<&'static str> {
        Some("{0} carpeta")
    }
    fn many_folders(&self) -> Option<&'static str> {
        Some("{0} carpetas")
    }
    fn one_file(&self) -> Option<&'static str> {
        Some("{0} archivo")
    }
    fn many_files(&self) -> Option<&'static str> {
        Some("{0} archivos")
    }
}
