//! The Splash view tree: title bar, filter rail, table and detail panel.
//!
//! Structure only. Every value shown here is either a fixed label or is
//! written by `app.rs`/`chrome.rs` from `data::Data`. Colours, spacing,
//! radii and type all come from the theme tokens.

use makepad_widgets::*;

use crate::app::App;

script_mod! {
    use mod.prelude.widgets.*
    use mod.widgets.*

    startup() do #(App::script_component(vm)){
        ui: Root{
            main_window := Window{
                window.title: "CMDB Viewer — synthetic fleet"
                window.inner_size: vec2(1440, 900)
                pass.clear_color: theme.color_bg_app
                body +: {
                    width: Fill height: Fill
                    flow: Down spacing: 0
                    padding: 0
                    margin: 0

                    // ---- title bar -------------------------------------
                    SolidView{
                        width: Fill height: 96
                        flow: Right spacing: theme.space_5
                        padding: Inset{left: theme.space_6, right: theme.space_6, top: theme.space_5, bottom: theme.space_5}
                        align: Align{y: 0.5}
                        draw_bg.color: theme.color_bg_container

                        View{
                            width: Fill height: Fit
                            flow: Down spacing: theme.space_1
                            Label{
                                height: Fit
                                text: "CMDB Viewer"
                                draw_text.color: theme.color_white
                                draw_text.text_style: theme.font_title_m
                            }
                            summary := Label{
                                height: Fit
                                max_lines: 1
                                text_overflow: Ellipsis
                                text: "3 accounts · 24 resources · 2 regions · 4 kinds · synthetic fixture"
                                draw_text.color: theme.color_text_meta
                                draw_text.text_style: theme.font_body_s
                            }
                        }
                        StatusChip{}
                    }
                    SolidView{width: Fill height: 1 draw_bg.color: theme.color_u_1}

                    // ---- rail / table / detail -------------------------
                    View{
                        width: Fill height: Fill
                        flow: Right spacing: 0

                        // left filter rail
                        SolidView{
                            width: 184 height: Fill
                            flow: Down spacing: theme.space_1
                            padding: Inset{left: theme.space_4, right: theme.space_3, top: theme.space_5, bottom: theme.space_4}
                            draw_bg.color: theme.color_bg_container

                            SectionLabel{text: "ACCOUNTS"}
                            View{
                                width: Fill height: 26 flow: Right spacing: 4 align: Align{y: 0.5}
                                all_mark := RailMark{}
                                all_button := FilterChip{text: "All"}
                            }
                            View{
                                width: Fill height: 26 flow: Right spacing: 4 align: Align{y: 0.5}
                                prod_mark := RailMark{}
                                prod_button := FilterChip{text: "prod"}
                            }
                            View{
                                width: Fill height: 26 flow: Right spacing: 4 align: Align{y: 0.5}
                                staging_mark := RailMark{}
                                staging_button := FilterChip{text: "staging"}
                            }
                            View{
                                width: Fill height: 26 flow: Right spacing: 4 align: Align{y: 0.5}
                                sandbox_mark := RailMark{}
                                sandbox_button := FilterChip{text: "sandbox"}
                            }

                            View{width: Fill height: theme.space_3}
                            SectionLabel{text: "REGIONS"}
                            View{
                                width: Fill height: 26 flow: Right spacing: 4 align: Align{y: 0.5}
                                useast_mark := RailMark{}
                                useast_button := FilterChip{text: "us-east-1"}
                            }
                            View{
                                width: Fill height: 26 flow: Right spacing: 4 align: Align{y: 0.5}
                                euwest_mark := RailMark{}
                                euwest_button := FilterChip{text: "eu-west-1"}
                            }

                            View{width: Fill height: theme.space_3}
                            SectionLabel{text: "KINDS"}
                            View{
                                width: Fill height: 26 flow: Right spacing: 4 align: Align{y: 0.5}
                                ec2_mark := RailMark{}
                                ec2_button := FilterChip{text: "ec2"}
                                ec2_count := Label{width: 16 height: Fit text: "" draw_text.color: theme.color_text_meta draw_text.text_style: theme.font_code}
                            }
                            View{
                                width: Fill height: 26 flow: Right spacing: 4 align: Align{y: 0.5}
                                s3_mark := RailMark{}
                                s3_button := FilterChip{text: "s3"}
                                s3_count := Label{width: 16 height: Fit text: "" draw_text.color: theme.color_text_meta draw_text.text_style: theme.font_code}
                            }
                            View{
                                width: Fill height: 26 flow: Right spacing: 4 align: Align{y: 0.5}
                                rds_mark := RailMark{}
                                rds_button := FilterChip{text: "rds"}
                                rds_count := Label{width: 16 height: Fit text: "" draw_text.color: theme.color_text_meta draw_text.text_style: theme.font_code}
                            }
                            View{
                                width: Fill height: 26 flow: Right spacing: 4 align: Align{y: 0.5}
                                lambda_mark := RailMark{}
                                lambda_button := FilterChip{text: "lambda"}
                                lambda_count := Label{width: 16 height: Fit text: "" draw_text.color: theme.color_text_meta draw_text.text_style: theme.font_code}
                            }

                            Filler{}
                            Label{
                                width: Fill height: Fit
                                text: "Embedded fixture · no AWS calls"
                                draw_text.color: theme.color_text_placeholder
                                draw_text.text_style: theme.font_body_s
                            }
                        }

                        // central resource table
                        SolidView{
                            width: Fill height: Fill
                            flow: Down spacing: 0
                            draw_bg.color: theme.color_bg_app

                            View{
                                width: Fill height: 56
                                flow: Right spacing: theme.space_2
                                padding: Inset{left: theme.space_6, right: theme.space_5, top: theme.space_3, bottom: theme.space_2}
                                align: Align{y: 0.5}
                                Label{
                                    height: Fit
                                    text: "Resources"
                                    draw_text.color: theme.color_text_hl
                                    draw_text.text_style: theme.font_title_s
                                }
                                table_status := Label{
                                    height: Fit
                                    max_lines: 1
                                    text_overflow: Ellipsis
                                    text: ""
                                    draw_text.color: theme.color_text_meta
                                    draw_text.text_style: theme.font_body_s
                                }
                                Filler{}
                                table_tab := ViewTab{text: "Table"}
                                graph_tab := ViewTab{text: "Graph"}
                                search_input := TextInput{
                                    width: 180 height: 28
                                    padding: Inset{left: 10 right: 10 top: 6 bottom: 0}
                                    label_align: Align{y: 0.5}
                                    empty_text: "Filter id or name"
                                    draw_bg +: {
                                        color: theme.color_bg_container
                                        color_hover: theme.color_bg_container
                                        color_focus: theme.color_bg_app
                                        color_empty: theme.color_bg_container
                                        border_color: theme.color_u_1
                                        border_color_hover: theme.color_u_2
                                        border_color_focus: theme.color_focus
                                        border_color_empty: theme.color_u_1
                                        border_radius: theme.radius_s
                                    }
                                    draw_text +: {
                                        color: theme.color_text
                                        color_hover: theme.color_text
                                        color_focus: theme.color_text
                                        color_down: theme.color_text
                                        color_empty: theme.color_text_placeholder
                                        color_empty_hover: theme.color_text_meta
                                        color_empty_focus: theme.color_text_meta
                                    }
                                    draw_cursor +: {color: theme.color_text}
                                }
                            }

                            table_pane := View{
                                width: Fill height: Fill
                                flow: Down spacing: 0

                            SolidView{
                                width: Fill height: 34
                                flow: Right spacing: 0
                                padding: Inset{left: theme.space_6, right: theme.space_6, top: 0 bottom: 0}
                                align: Align{y: 0.5}
                                draw_bg.color: theme.color_bg_container
                                ColumnHead{width: mod.widgets.cmdb_col_account text: "ACCOUNT"}
                                ColumnHead{width: mod.widgets.cmdb_col_region text: "REGION"}
                                ColumnHead{width: mod.widgets.cmdb_col_kind text: "KIND"}
                                View{
                                    width: Fill height: Fit
                                    padding: Inset{right: 12}
                                    ColumnHead{width: Fill text: "ID"}
                                }
                                ColumnHead{width: Fill text: "NAME"}
                            }
                            SolidView{width: Fill height: 1 draw_bg.color: theme.color_u_1}

                            View{
                                width: Fill height: Fill
                                table := mod.widgets.ResourceTable{}
                            }
                            }

                            graph_pane := View{
                                width: Fill height: Fill
                                visible: false
                                flow: Down spacing: theme.space_2
                                padding: Inset{left: theme.space_6, right: theme.space_6, top: theme.space_6, bottom: theme.space_6}
                                graph_host := View{
                                    width: Fill height: Fill
                                    flow: Down spacing: theme.space_2
                                    graph_counts := Label{
                                        height: Fit
                                        text: ""
                                        draw_text.color: theme.color_text_meta
                                        draw_text.text_style: theme.font_body_s
                                    }
                                    dag := mod.widgets.DagView{
                                        width: Fill height: Fill
                                    }
                                }
                            }
                        }

                        // right detail panel
                        SolidView{
                            width: 240 height: Fill
                            flow: Down spacing: theme.space_2
                            padding: Inset{left: theme.space_4, right: theme.space_4, top: theme.space_5, bottom: theme.space_4}
                            draw_bg.color: theme.color_bg_container

                            SectionLabel{text: "DETAIL"}
                            SolidView{width: Fill height: 1 draw_bg.color: theme.color_u_1}

                            detail_body := mod.widgets.DetailFields{}

                            detail_empty := Label{
                                width: Fill height: Fit
                                visible: false
                                text: "No resource matches these filters."
                                draw_text.color: theme.color_text_meta
                                draw_text.text_style: theme.font_body_s
                            }

                            Filler{}
                            Label{
                                width: Fill height: Fit
                                text: "Embedded synthetic fixture · no AWS calls"
                                draw_text.color: theme.color_text_placeholder
                                draw_text.text_style: theme.font_body_s
                            }
                        }
                    }
                }
            }
        }
    }
}
