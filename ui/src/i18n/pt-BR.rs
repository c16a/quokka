//! Brazilian Portuguese, also served for Portuguese with no region. A string not here falls back to American English.

use super::Translation;

pub struct PtBr;

impl Translation for PtBr {
    fn language_tags(&self) -> &'static [&'static str] {
        &["pt-BR", "pt"]
    }

    fn native_name(&self) -> &'static str {
        "Português (Brasil)"
    }

    fn is_singular(&self, count: usize) -> bool {
        count <= 1
    }

    // albums.rs
    fn name(&self) -> Option<&'static str> {
        Some("Nome")
    }
    fn year(&self) -> Option<&'static str> {
        Some("Ano")
    }
    fn date_added(&self) -> Option<&'static str> {
        Some("Data de inclusão")
    }
    fn albums(&self) -> Option<&'static str> {
        Some("Álbuns")
    }
    fn no_albums_match_these_filters(&self) -> Option<&'static str> {
        Some("Nenhum álbum corresponde a estes filtros.")
    }
    fn no_albums_yet_choose_your_library(&self) -> Option<&'static str> {
        Some("Nenhum álbum ainda. Escolha a pasta da sua biblioteca em Ajustes.")
    }
    fn all_genres(&self) -> Option<&'static str> {
        Some("Todos os gêneros")
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
        Some("Nenhum artista ainda. Escolha a pasta da sua biblioteca em Ajustes.")
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
        Some("Sincronizando a lista de estações…")
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
        Some("Buscar")
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
        Some("Ajustes")
    }
    fn scanning(&self) -> Option<&'static str> {
        Some("Analisando…")
    }
    fn saved_the_tmdb_key(&self) -> Option<&'static str> {
        Some("Chave do TMDb salva.")
    }
    fn could_not_save_settings(&self) -> Option<&'static str> {
        Some("Não foi possível salvar os ajustes: {0}")
    }
    fn radio_station_count(&self) -> Option<&'static str> {
        Some("Rádio: {0} estações.")
    }
    fn scanning_folder(&self) -> Option<&'static str> {
        Some("Analisando {0}…")
    }
    fn scanning_folder_queued(&self) -> Option<&'static str> {
        Some("Analisando {0} (mais {1} na fila)…")
    }
    fn scan_summary(&self) -> Option<&'static str> {
        Some("{0}: {1} adicionados ou alterados, {2} inalterados, {3} removidos.")
    }
    fn matched_on_tmdb(&self) -> Option<&'static str> {
        Some("{0} associados no TMDb.")
    }
    fn tmdb_lookup_stopped(&self) -> Option<&'static str> {
        Some("A busca no TMDb parou ({0}); será tentada de novo na próxima análise.")
    }
    fn could_not_find_album(&self) -> Option<&'static str> {
        Some("Não foi possível encontrar o álbum {0}")
    }
    fn could_not_tune_in(&self) -> Option<&'static str> {
        Some("Não foi possível sintonizar: {0}")
    }
    fn could_not_tune_out(&self) -> Option<&'static str> {
        Some("Não foi possível parar de ouvir: {0}")
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
        Some("Nenhum filme ainda. Escolha a pasta da sua biblioteca em Ajustes.")
    }
    fn movie(&self) -> Option<&'static str> {
        Some("FILME")
    }
    fn length_unknown(&self) -> Option<&'static str> {
        Some("Duração desconhecida")
    }
    fn start_over(&self) -> Option<&'static str> {
        Some("Começar de novo")
    }
    fn wrong_movie_choose_another(&self) -> Option<&'static str> {
        Some("Filme errado? Escolha outro")
    }
    fn remove_details(&self) -> Option<&'static str> {
        Some("Remover detalhes")
    }
    fn file_information(&self) -> Option<&'static str> {
        Some("Informações do arquivo")
    }
    fn source(&self) -> Option<&'static str> {
        Some("ORIGEM")
    }
    fn finding_details(&self) -> Option<&'static str> {
        Some("Buscando detalhes…")
    }
    fn find_details_on_tmdb(&self) -> Option<&'static str> {
        Some("Buscar detalhes no TMDb")
    }
    fn add_your_tmdb_api_key_in(&self) -> Option<&'static str> {
        Some("Adicione sua chave de API do TMDb em Ajustes para ver a sinopse e o elenco.")
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
        Some("Roteiro")
    }
    fn crew(&self) -> Option<&'static str> {
        Some("Equipe")
    }
    fn which_movie_is_this(&self) -> Option<&'static str> {
        Some("Que filme é este?")
    }
    fn pick_the_one_your_file_is(&self) -> Option<&'static str> {
        Some("Escolha o que corresponde ao seu arquivo. Nada é salvo até você escolher.")
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
        Some("a busca parou inesperadamente")
    }
    fn no_tmdb_details_for_this_movie(&self) -> Option<&'static str> {
        Some("Nenhum detalhe do TMDb para este filme.")
    }
    fn unknown(&self) -> Option<&'static str> {
        Some("Desconhecido")
    }
    fn container(&self) -> Option<&'static str> {
        Some("Contêiner")
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
        Some("Proporção")
    }
    fn frame_rate(&self) -> Option<&'static str> {
        Some("Taxa de quadros")
    }
    fn length(&self) -> Option<&'static str> {
        Some("Duração")
    }
    fn file_size(&self) -> Option<&'static str> {
        Some("Tamanho do arquivo")
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
        Some("Não foi possível conectar à estação.")
    }
    fn the_station_keeps_dropping_the_connection(&self) -> Option<&'static str> {
        Some("A estação continua derrubando a conexão. Pressione reproduzir para tentar de novo.")
    }
    fn pause(&self) -> Option<&'static str> {
        Some("Pausar")
    }
    fn connecting(&self) -> Option<&'static str> {
        Some("Conectando…")
    }
    fn paused(&self) -> Option<&'static str> {
        Some("Pausado")
    }
    fn live(&self) -> Option<&'static str> {
        Some("Ao vivo")
    }
    fn no_audio_output_device_available_2(&self) -> Option<&'static str> {
        Some("Nenhum dispositivo de saída de áudio disponível")
    }
    fn back_10_seconds_left_arrow(&self) -> Option<&'static str> {
        Some("Voltar 10 segundos (seta para a esquerda)")
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
        Some("Sair da tela cheia (Esc)")
    }
    fn full_screen_f(&self) -> Option<&'static str> {
        Some("Tela cheia (F)")
    }
    fn unmute(&self) -> Option<&'static str> {
        Some("Ativar som")
    }
    fn mute(&self) -> Option<&'static str> {
        Some("Silenciar")
    }
    fn loading_video(&self) -> Option<&'static str> {
        Some("Carregando vídeo…")
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
        Some("Nada tocando")
    }
    fn output(&self) -> Option<&'static str> {
        Some("Saída")
    }
    fn system_default(&self) -> Option<&'static str> {
        Some("Padrão do sistema")
    }
    fn up_next(&self) -> Option<&'static str> {
        Some("A seguir")
    }
    fn lyrics(&self) -> Option<&'static str> {
        Some("Letra")
    }
    fn playback_speed_click_to_change_speeds(&self) -> Option<&'static str> {
        Some(
            "Velocidade de reprodução. Clique para mudar. Velocidades diferentes de 1× também alteram o tom.",
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
        Some("Nenhuma playlist ainda. Adicione faixas de um álbum ou dos resultados da busca.")
    }
    fn rename(&self) -> Option<&'static str> {
        Some("Renomear")
    }
    fn delete_playlist(&self) -> Option<&'static str> {
        Some("Apagar playlist")
    }
    fn all_playlists(&self) -> Option<&'static str> {
        Some("Todas as playlists")
    }
    fn save(&self) -> Option<&'static str> {
        Some("Salvar")
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
        Some("As faixas que você colocar na fila aparecerão aqui.")
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
        Some("Nenhuma playlist ainda")
    }
    // radio.rs
    fn no_stations_yet(&self) -> Option<&'static str> {
        Some("Nenhuma estação ainda.")
    }
    fn sync_now(&self) -> Option<&'static str> {
        Some("Sincronizar agora")
    }
    fn syncing(&self) -> Option<&'static str> {
        Some("Sincronizando…")
    }
    fn your_stations(&self) -> Option<&'static str> {
        Some("Suas estações")
    }
    fn find_a_station_below_and_tune(&self) -> Option<&'static str> {
        Some("Encontre uma estação abaixo e sintonize. Ela aparecerá aqui.")
    }
    fn find_stations(&self) -> Option<&'static str> {
        Some("Encontrar estações")
    }
    fn sync_the_station_list_to_search(&self) -> Option<&'static str> {
        Some("Sincronize a lista de estações para buscar nela.")
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
        Some("PAUSADO")
    }
    fn live_now(&self) -> Option<&'static str> {
        Some("AO VIVO")
    }
    fn tune_out(&self) -> Option<&'static str> {
        Some("Parar de ouvir")
    }
    fn search_by_name_country_language_or(&self) -> Option<&'static str> {
        Some("Buscar por nome, país, idioma ou gênero")
    }
    fn internet_radio(&self) -> Option<&'static str> {
        Some("Rádio pela internet")
    }
    fn just_now(&self) -> Option<&'static str> {
        Some("agora mesmo")
    }
    // recent.rs
    fn recently_added(&self) -> Option<&'static str> {
        Some("Adicionados recentemente")
    }
    fn albums_you_add_to_the_library(&self) -> Option<&'static str> {
        Some("Os álbuns que você adiciona à biblioteca aparecem aqui.")
    }
    fn recently_played(&self) -> Option<&'static str> {
        Some("Tocados recentemente")
    }
    fn albums_you_play_appear_here(&self) -> Option<&'static str> {
        Some("Os álbuns que você toca aparecem aqui.")
    }
    // search.rs
    fn search_albums_songs_playlists_and_stations(&self) -> Option<&'static str> {
        Some("Buscar álbuns, músicas, playlists e estações")
    }
    fn search_for_an_album_a_song(&self) -> Option<&'static str> {
        Some(
            "Busque um álbum, uma música, uma playlist ou uma estação. Filtros: artist:, album:, genre:, year:, format:flac.",
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
        Some("Analisar de novo")
    }
    fn holds(&self) -> Option<&'static str> {
        Some("Contém:")
    }
    fn library(&self) -> Option<&'static str> {
        Some("Biblioteca")
    }
    fn no_folders_yet(&self) -> Option<&'static str> {
        Some("Nenhuma pasta ainda.")
    }
    fn rescan_all(&self) -> Option<&'static str> {
        Some("Analisar tudo de novo")
    }
    fn processing(&self) -> Option<&'static str> {
        Some("Processando")
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
        Some("Cada pasta contém um só tipo de mídia. Outros tipos de arquivo nela são ignorados.")
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
            "Ajusta as faixas para a mesma intensidade sonora. Arquivos com tags ReplayGain as usam; os demais são medidos em segundo plano enquanto tocam e mantêm seu próprio nível até lá.",
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
        Some("Transição gradual")
    }
    fn each_track_fades_into_the_next(&self) -> Option<&'static str> {
        Some(
            "Cada faixa se mistura à seguinte ao longo desta quantidade de segundos. Zero desativa.",
        )
    }
    fn look_up_synced_lyrics_on_lrclib(&self) -> Option<&'static str> {
        Some("Buscar letras sincronizadas no lrclib.net")
    }
    fn for_tracks_with_no_lrc_file(&self) -> Option<&'static str> {
        Some(
            "Para faixas sem um arquivo .lrc ao lado. Envia o título, o artista, o álbum e a duração de cada faixa ao lrclib.net. As letras encontradas ficam guardadas na biblioteca. Um arquivo .lrc sempre tem prioridade.",
        )
    }
    fn movie_details(&self) -> Option<&'static str> {
        Some("Detalhes dos filmes")
    }
    fn descriptions_and_cast_for_movies_come(&self) -> Option<&'static str> {
        Some(
            "As sinopses e os elencos dos filmes vêm do TMDb. Digite sua própria chave de API do TMDb para usá-los. A chave fica salva no seu arquivo de ajustes, legível apenas pela sua conta de usuário, e só é enviada ao TMDb.",
        )
    }
    fn tmdb_api_key(&self) -> Option<&'static str> {
        Some("Chave de API do TMDb")
    }
    fn save_key(&self) -> Option<&'static str> {
        Some("Salvar chave")
    }
    fn remove_key(&self) -> Option<&'static str> {
        Some("Remover chave")
    }
    fn a_key_is_saved(&self) -> Option<&'static str> {
        Some("Há uma chave salva.")
    }
    fn no_key_saved_movies_show_no(&self) -> Option<&'static str> {
        Some(
            "Nenhuma chave salva. Os filmes não mostram sinopse nem elenco até você adicionar uma.",
        )
    }
    fn this_product_uses_the_tmdb_api(&self) -> Option<&'static str> {
        Some("Este produto usa a API do TMDB, mas não é endossado nem certificado pelo TMDB.")
    }
    // side.rs
    fn nothing_playing_2(&self) -> Option<&'static str> {
        Some("Nada tocando.")
    }
    fn this_track_has_no_lyrics_in(&self) -> Option<&'static str> {
        Some("Esta faixa não tem letra nas tags nem um arquivo .lrc ao lado.")
    }
    fn looking_up_lyrics_on_lrclib_net(&self) -> Option<&'static str> {
        Some("Buscando a letra no lrclib.net…")
    }
    fn no_lyrics_were_found_on_lrclib(&self) -> Option<&'static str> {
        Some("Nenhuma letra encontrada no lrclib.net para esta faixa.")
    }
    fn try_again(&self) -> Option<&'static str> {
        Some("Tentar de novo")
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
        Some("Arquivo")
    }
    fn reveal_in_finder(&self) -> Option<&'static str> {
        Some("Mostrar no Finder")
    }
    fn tags(&self) -> Option<&'static str> {
        Some("Tags")
    }
    fn revert(&self) -> Option<&'static str> {
        Some("Reverter")
    }
    fn put_back_the_tags_the_file(&self) -> Option<&'static str> {
        Some("Restaurar as tags que o arquivo tinha antes da primeira edição")
    }
    fn look_up_synced_lyrics_on_lrclib_2(&self) -> Option<&'static str> {
        Some(
            "Buscar letras sincronizadas no lrclib.net? O título, o artista, o álbum e a duração de cada faixa que você abrir são enviados para lá.",
        )
    }
    fn turn_on_lyrics_lookup(&self) -> Option<&'static str> {
        Some("Ativar busca de letras")
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
        Some("Continuar em {0}")
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
        Some("Não foi possível conectar à estação: {0}")
    }
    fn could_not_switch_output(&self) -> Option<&'static str> {
        Some("Não foi possível trocar a saída: {0}")
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
        Some("Nenhum resultado para “{0}”.")
    }
    fn couldnt_reach_lrclib(&self) -> Option<&'static str> {
        Some("Não foi possível acessar o lrclib.net: {0}")
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
        Some("{0} arquivo")
    }
    fn many_files(&self) -> Option<&'static str> {
        Some("{0} arquivos")
    }
}
