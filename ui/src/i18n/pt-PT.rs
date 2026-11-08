//! European Portuguese, also served for Portuguese in Africa and Timor-Leste. A string not here falls back to American English.

use super::Translation;

pub struct PtPt;

impl Translation for PtPt {
    fn language_tags(&self) -> &'static [&'static str] {
        &[
            "pt-PT", "pt-AO", "pt-MZ", "pt-CV", "pt-GW", "pt-ST", "pt-TL",
        ]
    }

    fn native_name(&self) -> &'static str {
        "Português (Portugal)"
    }

    // albums.rs
    fn name(&self) -> Option<&'static str> {
        Some("Nome")
    }
    fn year(&self) -> Option<&'static str> {
        Some("Ano")
    }
    fn date_added(&self) -> Option<&'static str> {
        Some("Data de adição")
    }
    fn albums(&self) -> Option<&'static str> {
        Some("Álbuns")
    }
    fn no_albums_match_these_filters(&self) -> Option<&'static str> {
        Some("Nenhum álbum corresponde a estes filtros.")
    }
    fn no_albums_yet_choose_your_library(&self) -> Option<&'static str> {
        Some("Ainda não há álbuns. Escolha a pasta da sua biblioteca nas Definições.")
    }
    fn all_genres(&self) -> Option<&'static str> {
        Some("Todos os géneros")
    }
    fn all_years(&self) -> Option<&'static str> {
        Some("Todos os anos")
    }
    fn all_albums(&self) -> Option<&'static str> {
        Some("Todos os álbuns")
    }
    fn unknown_album(&self) -> Option<&'static str> {
        Some("Álbum desconhecido")
    }
    fn play(&self) -> Option<&'static str> {
        Some("Reproduzir")
    }
    fn shuffle(&self) -> Option<&'static str> {
        Some("Aleatório")
    }
    fn remove_from_favourites(&self) -> Option<&'static str> {
        Some("Remover dos favoritos")
    }
    fn add_to_favourites(&self) -> Option<&'static str> {
        Some("Adicionar aos favoritos")
    }
    fn album(&self) -> Option<&'static str> {
        Some("ÁLBUM")
    }
    // artists.rs
    fn no_artists_yet_choose_your_library(&self) -> Option<&'static str> {
        Some("Ainda não há artistas. Escolha a pasta da sua biblioteca nas Definições.")
    }
    fn artists(&self) -> Option<&'static str> {
        Some("Artistas")
    }
    fn all_artists(&self) -> Option<&'static str> {
        Some("Todos os artistas")
    }
    // format.rs
    fn unavailable(&self) -> Option<&'static str> {
        Some("Indisponível")
    }
    fn mono(&self) -> Option<&'static str> {
        Some("Mono")
    }
    fn stereo(&self) -> Option<&'static str> {
        Some("Estéreo")
    }
    // lib.rs
    fn add_a_folder_to_get_started(&self) -> Option<&'static str> {
        Some("Adicione uma pasta para começar.")
    }
    fn syncing_the_radio_station_list(&self) -> Option<&'static str> {
        Some("A sincronizar a lista de estações…")
    }
    fn radio_station_sync_stopped_unexpectedly(&self) -> Option<&'static str> {
        Some("A sincronização das estações parou inesperadamente.")
    }
    fn the_scan_stopped_unexpectedly(&self) -> Option<&'static str> {
        Some("a análise parou inesperadamente")
    }
    fn video(&self) -> Option<&'static str> {
        Some("Vídeo")
    }
    fn search(&self) -> Option<&'static str> {
        Some("Pesquisar")
    }
    fn recent(&self) -> Option<&'static str> {
        Some("Recentes")
    }
    fn playlists(&self) -> Option<&'static str> {
        Some("Playlists")
    }
    fn radio(&self) -> Option<&'static str> {
        Some("Rádio")
    }
    fn movies(&self) -> Option<&'static str> {
        Some("Filmes")
    }
    fn settings(&self) -> Option<&'static str> {
        Some("Definições")
    }
    fn scanning(&self) -> Option<&'static str> {
        Some("A analisar…")
    }
    fn saved_the_tmdb_key(&self) -> Option<&'static str> {
        Some("Chave do TMDb guardada.")
    }
    fn could_not_save_settings(&self) -> Option<&'static str> {
        Some("Não foi possível guardar as definições: {0}")
    }
    fn radio_station_count(&self) -> Option<&'static str> {
        Some("Rádio: {0} estações.")
    }
    fn scanning_folder(&self) -> Option<&'static str> {
        Some("A analisar {0}…")
    }
    fn scanning_folder_queued(&self) -> Option<&'static str> {
        Some("A analisar {0} (mais {1} na fila)…")
    }
    fn scan_summary(&self) -> Option<&'static str> {
        Some("{0}: {1} adicionados ou alterados, {2} inalterados, {3} removidos.")
    }
    fn matched_on_tmdb(&self) -> Option<&'static str> {
        Some("{0} associados no TMDb.")
    }
    fn tmdb_lookup_stopped(&self) -> Option<&'static str> {
        Some("A pesquisa no TMDb parou ({0}); será tentada novamente na próxima análise.")
    }
    fn could_not_find_album(&self) -> Option<&'static str> {
        Some("Não foi possível encontrar o álbum {0}")
    }
    fn could_not_tune_in(&self) -> Option<&'static str> {
        Some("Não foi possível sintonizar: {0}")
    }
    fn could_not_tune_out(&self) -> Option<&'static str> {
        Some("Não foi possível deixar de ouvir: {0}")
    }
    fn could_not_add_to_playlist(&self) -> Option<&'static str> {
        Some("Não foi possível adicionar à playlist: {0}")
    }
    fn already_in_library(&self) -> Option<&'static str> {
        Some("{0} já está na biblioteca.")
    }
    fn removed_folder(&self) -> Option<&'static str> {
        Some("{0} removida ({1}).")
    }
    fn could_not_remove_folder(&self) -> Option<&'static str> {
        Some("Não foi possível remover {0}: {1}")
    }
    fn hide_sidebar(&self) -> Option<&'static str> {
        Some("Ocultar barra lateral")
    }
    fn show_sidebar(&self) -> Option<&'static str> {
        Some("Mostrar barra lateral")
    }
    // movies.rs
    fn no_movies_yet_choose_your_library(&self) -> Option<&'static str> {
        Some("Ainda não há filmes. Escolha a pasta da sua biblioteca nas Definições.")
    }
    fn movie(&self) -> Option<&'static str> {
        Some("FILME")
    }
    fn length_unknown(&self) -> Option<&'static str> {
        Some("Duração desconhecida")
    }
    fn start_over(&self) -> Option<&'static str> {
        Some("Começar do início")
    }
    fn wrong_movie_choose_another(&self) -> Option<&'static str> {
        Some("Filme errado? Escolha outro")
    }
    fn remove_details(&self) -> Option<&'static str> {
        Some("Remover detalhes")
    }
    fn file_information(&self) -> Option<&'static str> {
        Some("Informações do ficheiro")
    }
    fn source(&self) -> Option<&'static str> {
        Some("ORIGEM")
    }
    fn finding_details(&self) -> Option<&'static str> {
        Some("A procurar detalhes…")
    }
    fn find_details_on_tmdb(&self) -> Option<&'static str> {
        Some("Procurar detalhes no TMDb")
    }
    fn add_your_tmdb_api_key_in(&self) -> Option<&'static str> {
        Some("Adicione a sua chave de API do TMDb nas Definições para ver a sinopse e o elenco.")
    }
    fn synopsis(&self) -> Option<&'static str> {
        Some("Sinopse")
    }
    fn cast(&self) -> Option<&'static str> {
        Some("Elenco")
    }
    fn director(&self) -> Option<&'static str> {
        Some("Direção")
    }
    fn writer(&self) -> Option<&'static str> {
        Some("Argumento")
    }
    fn crew(&self) -> Option<&'static str> {
        Some("Equipe")
    }
    fn which_movie_is_this(&self) -> Option<&'static str> {
        Some("Que filme é este?")
    }
    fn pick_the_one_your_file_is(&self) -> Option<&'static str> {
        Some("Escolha o que corresponde ao seu ficheiro. Nada é guardado até escolher.")
    }
    fn none_of_these(&self) -> Option<&'static str> {
        Some("Nenhum destes")
    }
    fn left_unmatched(&self) -> Option<&'static str> {
        Some("Deixado sem correspondência.")
    }
    fn this_one(&self) -> Option<&'static str> {
        Some("Este")
    }
    fn the_lookup_stopped_unexpectedly(&self) -> Option<&'static str> {
        Some("a pesquisa parou inesperadamente")
    }
    fn no_tmdb_details_for_this_movie(&self) -> Option<&'static str> {
        Some("Nenhum detalhe do TMDb para este filme.")
    }
    fn unknown(&self) -> Option<&'static str> {
        Some("Desconhecido")
    }
    fn container(&self) -> Option<&'static str> {
        Some("Contentor")
    }
    fn codec(&self) -> Option<&'static str> {
        Some("Codec")
    }
    fn colour_depth(&self) -> Option<&'static str> {
        Some("Profundidade de cor")
    }
    fn chroma_subsampling(&self) -> Option<&'static str> {
        Some("Subamostragem de croma")
    }
    fn dynamic_range(&self) -> Option<&'static str> {
        Some("Faixa dinâmica")
    }
    fn resolution(&self) -> Option<&'static str> {
        Some("Resolução")
    }
    fn aspect_ratio(&self) -> Option<&'static str> {
        Some("Formato")
    }
    fn frame_rate(&self) -> Option<&'static str> {
        Some("Fotogramas por segundo")
    }
    fn length(&self) -> Option<&'static str> {
        Some("Duração")
    }
    fn file_size(&self) -> Option<&'static str> {
        Some("Tamanho do ficheiro")
    }
    fn mpeg_transport_stream(&self) -> Option<&'static str> {
        Some("Fluxo de transporte MPEG")
    }
    fn mpeg_program_stream(&self) -> Option<&'static str> {
        Some("Fluxo de programa MPEG")
    }
    fn raw_av1_stream_obu(&self) -> Option<&'static str> {
        Some("Fluxo AV1 bruto (OBU)")
    }
    // playback.rs
    fn no_audio_output_device_available(&self) -> Option<&'static str> {
        Some("nenhum dispositivo de saída de áudio disponível")
    }
    fn could_not_connect_to_the_station(&self) -> Option<&'static str> {
        Some("Não foi possível ligar à estação.")
    }
    fn the_station_keeps_dropping_the_connection(&self) -> Option<&'static str> {
        Some("A estação continua a perder a ligação. Prima reproduzir para tentar novamente.")
    }
    fn pause(&self) -> Option<&'static str> {
        Some("Pausar")
    }
    fn connecting(&self) -> Option<&'static str> {
        Some("A ligar…")
    }
    fn paused(&self) -> Option<&'static str> {
        Some("Em pausa")
    }
    fn live(&self) -> Option<&'static str> {
        Some("Em direto")
    }
    fn no_audio_output_device_available_2(&self) -> Option<&'static str> {
        Some("Nenhum dispositivo de saída de áudio disponível")
    }
    fn back_10_seconds_left_arrow(&self) -> Option<&'static str> {
        Some("Recuar 10 segundos (seta para a esquerda)")
    }
    fn pause_space(&self) -> Option<&'static str> {
        Some("Pausar (Espaço)")
    }
    fn play_space(&self) -> Option<&'static str> {
        Some("Reproduzir (Espaço)")
    }
    fn forward_10_seconds_right_arrow(&self) -> Option<&'static str> {
        Some("Avançar 10 segundos (seta para a direita)")
    }
    fn exit_full_screen_escape(&self) -> Option<&'static str> {
        Some("Sair do ecrã completo (Esc)")
    }
    fn full_screen_f(&self) -> Option<&'static str> {
        Some("Ecrã completo (F)")
    }
    fn unmute(&self) -> Option<&'static str> {
        Some("Ativar som")
    }
    fn mute(&self) -> Option<&'static str> {
        Some("Silenciar")
    }
    fn loading_video(&self) -> Option<&'static str> {
        Some("A carregar o vídeo…")
    }
    fn previous_track(&self) -> Option<&'static str> {
        Some("Faixa anterior")
    }
    fn next_track(&self) -> Option<&'static str> {
        Some("Próxima faixa")
    }
    fn repeat(&self) -> Option<&'static str> {
        Some("Repetir")
    }
    fn nothing_playing(&self) -> Option<&'static str> {
        Some("Nada em reprodução")
    }
    fn output(&self) -> Option<&'static str> {
        Some("Saída")
    }
    fn system_default(&self) -> Option<&'static str> {
        Some("Predefinição do sistema")
    }
    fn up_next(&self) -> Option<&'static str> {
        Some("A seguir")
    }
    fn lyrics(&self) -> Option<&'static str> {
        Some("Letra")
    }
    fn playback_speed_click_to_change_speeds(&self) -> Option<&'static str> {
        Some(
            "Velocidade de reprodução. Clique para alterar. Velocidades diferentes de 1× também alteram o tom.",
        )
    }
    fn track_info(&self) -> Option<&'static str> {
        Some("Informações da faixa")
    }
    // playlists.rs
    fn new_playlist_name(&self) -> Option<&'static str> {
        Some("Nome da nova playlist")
    }
    fn create(&self) -> Option<&'static str> {
        Some("Criar")
    }
    fn no_playlists_yet_add_tracks_from(&self) -> Option<&'static str> {
        Some(
            "Ainda não há playlists. Adicione faixas a partir de um álbum ou dos resultados da pesquisa.",
        )
    }
    fn rename(&self) -> Option<&'static str> {
        Some("Renomear")
    }
    fn delete_playlist(&self) -> Option<&'static str> {
        Some("Eliminar playlist")
    }
    fn all_playlists(&self) -> Option<&'static str> {
        Some("Todas as playlists")
    }
    fn save(&self) -> Option<&'static str> {
        Some("Guardar")
    }
    fn cancel(&self) -> Option<&'static str> {
        Some("Cancelar")
    }
    fn play_playlist(&self) -> Option<&'static str> {
        Some("Reproduzir playlist")
    }
    fn shuffle_playlist(&self) -> Option<&'static str> {
        Some("Reproduzir playlist em ordem aleatória")
    }
    // queue.rs
    fn up_next_2(&self) -> Option<&'static str> {
        Some("A seguir")
    }
    fn nothing_queued(&self) -> Option<&'static str> {
        Some("Nada na fila")
    }
    fn clear(&self) -> Option<&'static str> {
        Some("Limpar")
    }
    fn tracks_you_queue_will_appear_here(&self) -> Option<&'static str> {
        Some("As faixas que colocar na fila aparecem aqui.")
    }
    fn play_next(&self) -> Option<&'static str> {
        Some("Reproduzir em seguida")
    }
    fn add_to_queue(&self) -> Option<&'static str> {
        Some("Adicionar à fila")
    }
    fn add_to_playlist(&self) -> Option<&'static str> {
        Some("Adicionar à playlist")
    }
    fn no_playlists_yet(&self) -> Option<&'static str> {
        Some("Ainda não há playlists")
    }
    // radio.rs
    fn no_stations_yet(&self) -> Option<&'static str> {
        Some("Ainda não há estações.")
    }
    fn sync_now(&self) -> Option<&'static str> {
        Some("Sincronizar agora")
    }
    fn syncing(&self) -> Option<&'static str> {
        Some("A sincronizar…")
    }
    fn your_stations(&self) -> Option<&'static str> {
        Some("As suas estações")
    }
    fn find_a_station_below_and_tune(&self) -> Option<&'static str> {
        Some("Encontre uma estação abaixo e sintonize-a. Vai aparecer aqui.")
    }
    fn find_stations(&self) -> Option<&'static str> {
        Some("Encontrar estações")
    }
    fn sync_the_station_list_to_search(&self) -> Option<&'static str> {
        Some("Sincronize a lista de estações para pesquisar nela.")
    }
    fn no_stations_match(&self) -> Option<&'static str> {
        Some("Nenhuma estação corresponde.")
    }
    fn country_any(&self) -> Option<&'static str> {
        Some("País: qualquer")
    }
    fn any_country(&self) -> Option<&'static str> {
        Some("Qualquer país")
    }
    fn codec_any(&self) -> Option<&'static str> {
        Some("Codec: qualquer")
    }
    fn any_codec(&self) -> Option<&'static str> {
        Some("Qualquer codec")
    }
    fn bitrate_any(&self) -> Option<&'static str> {
        Some("Taxa de bits: qualquer")
    }
    fn any_bitrate(&self) -> Option<&'static str> {
        Some("Qualquer taxa de bits")
    }
    fn most_popular_first(&self) -> Option<&'static str> {
        Some("Mais populares primeiro")
    }
    fn paused_2(&self) -> Option<&'static str> {
        Some("EM PAUSA")
    }
    fn live_now(&self) -> Option<&'static str> {
        Some("EM DIRETO")
    }
    fn tune_out(&self) -> Option<&'static str> {
        Some("Deixar de ouvir")
    }
    fn search_by_name_country_language_or(&self) -> Option<&'static str> {
        Some("Pesquisar por nome, país, idioma ou género")
    }
    fn internet_radio(&self) -> Option<&'static str> {
        Some("Rádio na Internet")
    }
    fn just_now(&self) -> Option<&'static str> {
        Some("agora mesmo")
    }
    // recent.rs
    fn recently_added(&self) -> Option<&'static str> {
        Some("Adicionados recentemente")
    }
    fn albums_you_add_to_the_library(&self) -> Option<&'static str> {
        Some("Os álbuns que adicionar à biblioteca aparecem aqui.")
    }
    fn recently_played(&self) -> Option<&'static str> {
        Some("Reproduzidos recentemente")
    }
    fn albums_you_play_appear_here(&self) -> Option<&'static str> {
        Some("Os álbuns que reproduzir aparecem aqui.")
    }
    // search.rs
    fn search_albums_songs_playlists_and_stations(&self) -> Option<&'static str> {
        Some("Pesquisar álbuns, músicas, playlists e estações")
    }
    fn search_for_an_album_a_song(&self) -> Option<&'static str> {
        Some(
            "Pesquise um álbum, uma música, uma playlist ou uma estação. Filtros: artist:, album:, genre:, year:, format:flac.",
        )
    }
    fn stations(&self) -> Option<&'static str> {
        Some("Estações")
    }
    fn songs(&self) -> Option<&'static str> {
        Some("Músicas")
    }
    // settings_view.rs
    fn music(&self) -> Option<&'static str> {
        Some("Música")
    }
    fn remove(&self) -> Option<&'static str> {
        Some("Remover")
    }
    fn rescan(&self) -> Option<&'static str> {
        Some("Analisar novamente")
    }
    fn holds(&self) -> Option<&'static str> {
        Some("Contém:")
    }
    fn library(&self) -> Option<&'static str> {
        Some("Biblioteca")
    }
    fn no_folders_yet(&self) -> Option<&'static str> {
        Some("Ainda não há pastas.")
    }
    fn rescan_all(&self) -> Option<&'static str> {
        Some("Analisar tudo novamente")
    }
    fn processing(&self) -> Option<&'static str> {
        Some("A processar")
    }
    fn kind_not_set(&self) -> Option<&'static str> {
        Some("Tipo não definido")
    }
    fn choose_what_each_of_these_folders(&self) -> Option<&'static str> {
        Some("Escolha o que cada uma destas pastas contém.")
    }
    fn tv_shows(&self) -> Option<&'static str> {
        Some("Séries")
    }
    fn coming_soon(&self) -> Option<&'static str> {
        Some("Em breve")
    }
    fn add_a_folder(&self) -> Option<&'static str> {
        Some("Adicionar uma pasta")
    }
    fn each_folder_holds_one_kind_of(&self) -> Option<&'static str> {
        Some(
            "Cada pasta contém um só tipo de conteúdo multimédia. Os outros tipos de ficheiro que contiver são ignorados.",
        )
    }
    fn tv_shows_soon(&self) -> Option<&'static str> {
        Some("Séries · Em breve")
    }
    fn choose_folder(&self) -> Option<&'static str> {
        Some("Escolher pasta…")
    }
    fn choose_a_folder_for_your_library(&self) -> Option<&'static str> {
        Some("Escolha uma pasta para sua biblioteca")
    }
    fn appearance(&self) -> Option<&'static str> {
        Some("Aparência")
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
        Some("Escuro")
    }
    fn light(&self) -> Option<&'static str> {
        Some("Claro")
    }
    fn system(&self) -> Option<&'static str> {
        Some("Sistema")
    }
    fn playback(&self) -> Option<&'static str> {
        Some("Reprodução")
    }
    fn loudness(&self) -> Option<&'static str> {
        Some("Volume uniforme")
    }
    fn levels_tracks_to_the_same_loudness(&self) -> Option<&'static str> {
        Some(
            "Ajusta as faixas para a mesma intensidade sonora. Os ficheiros com etiquetas ReplayGain usam-nas; os restantes são medidos em segundo plano durante a reprodução e mantêm o seu próprio nível até lá.",
        )
    }
    fn album_2(&self) -> Option<&'static str> {
        Some("Álbum")
    }
    fn track(&self) -> Option<&'static str> {
        Some("Faixa")
    }
    fn off(&self) -> Option<&'static str> {
        Some("Desativado")
    }
    fn crossfade(&self) -> Option<&'static str> {
        Some("Transição suave")
    }
    fn each_track_fades_into_the_next(&self) -> Option<&'static str> {
        Some("Cada faixa funde-se na seguinte ao longo deste número de segundos. Zero desativa.")
    }
    fn look_up_synced_lyrics_on_lrclib(&self) -> Option<&'static str> {
        Some("Procurar letras sincronizadas no lrclib.net")
    }
    fn for_tracks_with_no_lrc_file(&self) -> Option<&'static str> {
        Some(
            "Para faixas sem um ficheiro .lrc ao lado. Envia o título, o artista, o álbum e a duração de cada faixa para o lrclib.net. As letras encontradas ficam guardadas na biblioteca. Um ficheiro .lrc tem sempre prioridade.",
        )
    }
    fn movie_details(&self) -> Option<&'static str> {
        Some("Detalhes dos filmes")
    }
    fn descriptions_and_cast_for_movies_come(&self) -> Option<&'static str> {
        Some(
            "As sinopses e os elencos dos filmes vêm do TMDb. Introduza a sua própria chave de API do TMDb para os usar. A chave fica guardada no seu ficheiro de definições, legível apenas pela sua conta de utilizador, e só é enviada para o TMDb.",
        )
    }
    fn tmdb_api_key(&self) -> Option<&'static str> {
        Some("Chave de API do TMDb")
    }
    fn save_key(&self) -> Option<&'static str> {
        Some("Guardar chave")
    }
    fn remove_key(&self) -> Option<&'static str> {
        Some("Remover chave")
    }
    fn a_key_is_saved(&self) -> Option<&'static str> {
        Some("Há uma chave guardada.")
    }
    fn no_key_saved_movies_show_no(&self) -> Option<&'static str> {
        Some("Nenhuma chave guardada. Os filmes não mostram sinopse nem elenco até adicionar uma.")
    }
    fn this_product_uses_the_tmdb_api(&self) -> Option<&'static str> {
        Some("Este produto usa a API do TMDB, mas não é apoiado nem certificado pelo TMDB.")
    }
    // side.rs
    fn nothing_playing_2(&self) -> Option<&'static str> {
        Some("Nada em reprodução.")
    }
    fn this_track_has_no_lyrics_in(&self) -> Option<&'static str> {
        Some("Esta faixa não tem letra nas etiquetas nem um ficheiro .lrc ao lado.")
    }
    fn looking_up_lyrics_on_lrclib_net(&self) -> Option<&'static str> {
        Some("A procurar a letra no lrclib.net…")
    }
    fn no_lyrics_were_found_on_lrclib(&self) -> Option<&'static str> {
        Some("Nenhuma letra encontrada no lrclib.net para esta faixa.")
    }
    fn try_again(&self) -> Option<&'static str> {
        Some("Tentar novamente")
    }
    fn title(&self) -> Option<&'static str> {
        Some("Título")
    }
    fn artist(&self) -> Option<&'static str> {
        Some("Artista")
    }
    fn duration(&self) -> Option<&'static str> {
        Some("Duração")
    }
    fn size(&self) -> Option<&'static str> {
        Some("Tamanho")
    }
    fn bitrate(&self) -> Option<&'static str> {
        Some("Taxa de bits")
    }
    fn format(&self) -> Option<&'static str> {
        Some("Formato")
    }
    fn channels(&self) -> Option<&'static str> {
        Some("Canais")
    }
    fn pictures(&self) -> Option<&'static str> {
        Some("Imagens")
    }
    fn file(&self) -> Option<&'static str> {
        Some("Ficheiro")
    }
    fn reveal_in_finder(&self) -> Option<&'static str> {
        Some("Mostrar no Finder")
    }
    fn tags(&self) -> Option<&'static str> {
        Some("Etiquetas")
    }
    fn revert(&self) -> Option<&'static str> {
        Some("Reverter")
    }
    fn put_back_the_tags_the_file(&self) -> Option<&'static str> {
        Some("Repor as etiquetas que o ficheiro tinha antes da primeira edição")
    }
    fn look_up_synced_lyrics_on_lrclib_2(&self) -> Option<&'static str> {
        Some(
            "Procurar letras sincronizadas no lrclib.net? O título, o artista, o álbum e a duração de cada faixa que abrir são enviados para lá.",
        )
    }
    fn turn_on_lyrics_lookup(&self) -> Option<&'static str> {
        Some("Ativar pesquisa de letras")
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
        Some("Retomar em {0}")
    }
    fn could_not_fetch_details(&self) -> Option<&'static str> {
        Some("Não foi possível obter os detalhes: {0}")
    }
    fn fps_average_variable(&self) -> Option<&'static str> {
        Some("{0} fps em média, variável")
    }
    #[cfg(not(target_os = "macos"))]
    fn video_needs_libmpv(&self) -> Option<&'static str> {
        Some("O vídeo precisa da libmpv: {0}")
    }
    fn no_audio_output_error(&self) -> Option<&'static str> {
        Some("Sem saída de áudio: {0}")
    }
    fn could_not_play_station(&self) -> Option<&'static str> {
        Some("Não foi possível reproduzir a estação: {0}")
    }
    fn could_not_connect_station_error(&self) -> Option<&'static str> {
        Some("Não foi possível ligar à estação: {0}")
    }
    fn could_not_switch_output(&self) -> Option<&'static str> {
        Some("Não foi possível mudar a saída: {0}")
    }
    fn could_not_play_video(&self) -> Option<&'static str> {
        Some("Não foi possível reproduzir o vídeo: {0}")
    }
    fn playback_stopped_early(&self) -> Option<&'static str> {
        Some("A reprodução parou antes do fim: {0}")
    }
    fn could_not_play_item(&self) -> Option<&'static str> {
        Some("Não foi possível reproduzir {0}: {1}")
    }
    fn stations_in_directory(&self) -> Option<&'static str> {
        Some("{0} estações no diretório · sincronizado {1}")
    }
    fn tuned_in(&self) -> Option<&'static str> {
        Some("{0} sintonizadas")
    }
    fn tuned_in_suffix(&self) -> Option<&'static str> {
        Some("  ·  Sintonizada")
    }
    fn bitrate_at_least(&self) -> Option<&'static str> {
        Some("Taxa de bits: {0}+ kbps")
    }
    fn never(&self) -> Option<&'static str> {
        Some("nunca")
    }
    fn minutes_ago(&self) -> Option<&'static str> {
        Some("há {0} minutos")
    }
    fn hours_ago(&self) -> Option<&'static str> {
        Some("há {0} horas")
    }
    fn days_ago(&self) -> Option<&'static str> {
        Some("há {0} dias")
    }
    fn no_results_for(&self) -> Option<&'static str> {
        Some("Sem resultados para “{0}”.")
    }
    fn couldnt_reach_lrclib(&self) -> Option<&'static str> {
        Some("Não foi possível contactar o lrclib.net: {0}")
    }
    fn radio_sync_failed(&self) -> Option<&'static str> {
        Some("Falha na sincronização das estações: {0}")
    }
    fn could_not_be_scanned(&self) -> Option<&'static str> {
        Some("{0}: não foi possível analisar ({1}).")
    }
    fn one_movie(&self) -> Option<&'static str> {
        Some("{0} filme")
    }
    fn many_movies(&self) -> Option<&'static str> {
        Some("{0} filmes")
    }
    fn one_album(&self) -> Option<&'static str> {
        Some("{0} álbum")
    }
    fn many_albums(&self) -> Option<&'static str> {
        Some("{0} álbuns")
    }
    fn one_track(&self) -> Option<&'static str> {
        Some("{0} faixa")
    }
    fn many_tracks(&self) -> Option<&'static str> {
        Some("{0} faixas")
    }
    fn one_folder(&self) -> Option<&'static str> {
        Some("{0} pasta")
    }
    fn many_folders(&self) -> Option<&'static str> {
        Some("{0} pastas")
    }
    fn one_file(&self) -> Option<&'static str> {
        Some("{0} ficheiro")
    }
    fn many_files(&self) -> Option<&'static str> {
        Some("{0} ficheiros")
    }
}
