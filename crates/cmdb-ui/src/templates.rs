use makepad_widgets::*;

script_mod! {
    use mod.prelude.widgets.*
    use mod.widgets.*

    mod.widgets.StatusChip = RoundedView{
        width: Fit height: Fit
        flow: Right spacing: 6
        align: Align{y: 0.5}
        padding: Inset{left: 10 right: 10 top: 4 bottom: 4}
        draw_bg.color: theme.color_info_container
        draw_bg.border_radius: theme.radius_full
        RoundedView{
            width: 6 height: 6
            draw_bg.color: theme.color_on_info_container
            draw_bg.border_radius: theme.radius_full
        }
        Label{
            height: Fit
            text: "READ ONLY"
            draw_text.color: theme.color_on_info_container
            draw_text.text_style: theme.font_bold{font_size: theme.type_label_s_size}
        }
    }

    mod.widgets.SectionLabel = Label{
        height: Fit
        draw_text.color: theme.color_text_meta
        draw_text.text_style: theme.font_bold{font_size: theme.type_label_s_size}
    }

    mod.widgets.ColumnHead = Label{
        height: Fit
        draw_text.color: theme.color_text_meta
        draw_text.text_style: theme.font_bold{font_size: theme.type_label_s_size}
    }

    mod.widgets.FieldKey = Label{
        height: Fit
        draw_text.color: theme.color_text_meta
        draw_text.text_style: theme.font_bold{font_size: theme.type_label_s_size}
    }

    mod.widgets.FieldValue = Label{
        width: Fill height: Fit
        max_lines: 1
        text_overflow: Ellipsis
        draw_text.color: theme.color_text
        draw_text.text_style: theme.font_code
    }

    mod.widgets.RailMark = SolidView{
        width: 2 height: 18
        visible: false
        draw_bg.color: theme.color_focus
    }

    mod.widgets.FilterChip = ButtonFlatter{
        width: Fill height: 26
        align: Align{x: 0.0 y: 0.5}
        label_align: Align{x: 0.0 y: 0.5}
        padding: Inset{left: 2 right: 4 top: 0 bottom: 0}
        draw_text +: {
            color: theme.color_text
            color_hover: theme.color_white
            color_down: theme.color_white
            text_style +: {font_size: theme.font_size_p}
        }
    }

    mod.widgets.ViewTab = ButtonFlatter{
        width: Fit height: 24
        padding: Inset{left: 8 right: 8 top: 0 bottom: 0}
        align: Align{x: 0.5 y: 0.5}
        draw_text +: {
            color: theme.color_text_meta
            color_hover: theme.color_white
            color_down: theme.color_white
            text_style +: {font_size: theme.font_size_p}
        }
    }

    mod.widgets.DetailFields = View{
        width: Fill height: Fit
        flow: Down spacing: theme.space_3
        padding: Inset{top: theme.space_2}
        detail_name := Label{width: Fill height: Fit max_lines: 2 text_overflow: Ellipsis text: "" draw_text.color: theme.color_white draw_text.text_style: theme.font_title_s}
        View{
            width: Fill height: Fit flow: Down spacing: 1
            mod.widgets.FieldKey{text: "ACCOUNT"}
            detail_account := mod.widgets.FieldValue{text: ""}
        }
        View{
            width: Fill height: Fit flow: Down spacing: 1
            mod.widgets.FieldKey{text: "REGION"}
            detail_region := mod.widgets.FieldValue{text: ""}
        }
        View{
            width: Fill height: Fit flow: Down spacing: 1
            mod.widgets.FieldKey{text: "KIND"}
            detail_kind := mod.widgets.FieldValue{text: ""}
        }
        View{
            width: Fill height: Fit flow: Down spacing: 1
            mod.widgets.FieldKey{text: "ID"}
            detail_id := mod.widgets.FieldValue{text: ""}
        }
        View{
            width: Fill height: Fit flow: Down spacing: 1
            mod.widgets.FieldKey{text: "RECORD"}
            detail_record := mod.widgets.FieldValue{text: ""}
        }
        View{
            width: Fill height: Fit flow: Down spacing: 1
            mod.widgets.FieldKey{text: "CONTENT HASH"}
            detail_hash := mod.widgets.FieldValue{text: ""}
        }
    }
}
