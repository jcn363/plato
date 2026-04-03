#include <mupdf/fitz.h>
#include <mupdf/pdf/document.h>
#include <mupdf/pdf/page.h>
#include <mupdf/pdf/annot.h>
#include <mupdf/pdf/object.h>
#include <mupdf/pdf/resource.h>
#include <mupdf/pdf/xref.h>

#define WRAP(name, ret_type, failure_val, call, ...) \
    ret_type mp_##name(fz_context *ctx, ##__VA_ARGS__) { \
        ret_type ret; \
        fz_try (ctx) { ret = call; } \
        fz_catch (ctx) { ret = failure_val; } \
        return ret; \
    }

WRAP(open_document, fz_document*, NULL, fz_open_document(ctx, path), char *path)
WRAP(open_document_with_stream, fz_document*, NULL, fz_open_document_with_stream(ctx, kind, stream), const char *kind, fz_stream *stream)
WRAP(load_page, fz_page*, NULL, fz_load_page(ctx, doc, pageno), fz_document *doc, int pageno)
WRAP(load_outline, fz_outline*, NULL, fz_load_outline(ctx, doc), fz_document *doc)
WRAP(load_links, fz_link*, NULL, fz_load_links(ctx, page), fz_page *page)
WRAP(count_pages, int, -1, fz_count_pages(ctx, doc), fz_document *doc)
WRAP(page_number_from_location, int, -1, fz_page_number_from_location(ctx, doc, loc), fz_document *doc, fz_location loc)
WRAP(new_pixmap_from_page, fz_pixmap*, NULL, fz_new_pixmap_from_page(ctx, page, mat, cs, alpha), fz_page *page, fz_matrix mat, fz_colorspace *cs, int alpha)
WRAP(new_stext_page_from_page, fz_stext_page*, NULL, fz_new_stext_page_from_page(ctx, page, options), fz_page *page, fz_stext_options *options)

/* Type alias for annotation - MuPDF uses pdf_annot but Rust expects fz_annot */
typedef pdf_annot fz_annot;

/* Write options structure matching Rust FZWriteOptions */
typedef struct {
    int incremental;
    int clean;
    int garbage;
    int ascii;
    int linear;
} fz_write_options;

/* PDF document manipulation wrappers */

fz_document *fz_new_pdf_document(fz_context *ctx) {
    fz_document *ret = NULL;
    fz_try (ctx) {
        pdf_document *pdf = pdf_create_document(ctx);
        ret = (fz_document *)pdf;
    }
    fz_catch (ctx) {
        ret = NULL;
    }
    return ret;
}

int fz_pdf_count_pages(fz_context *ctx, fz_document *doc) {
    int ret = 0;
    fz_try (ctx) {
        ret = fz_count_pages(ctx, doc);
    }
    fz_catch (ctx) {
        ret = 0;
    }
    return ret;
}

int fz_pdf_can_move_pages(fz_context *ctx, fz_document *doc) {
    return 1;
}

void fz_pdf_move_page(fz_context *ctx, fz_document *doc, int src, int dst) {
    fz_try (ctx) {
        pdf_document *pdf = pdf_document_from_fz_document(ctx, doc);
        pdf_obj *page = pdf_lookup_page_obj(ctx, pdf, src);
        pdf_obj *page_ref = pdf_keep_obj(ctx, page);
        pdf_delete_page(ctx, pdf, src);
        int insert_at = (dst >= src) ? dst - 1 : dst;
        pdf_insert_page(ctx, pdf, insert_at, page_ref);
        pdf_drop_obj(ctx, page_ref);
    }
    fz_catch (ctx) {
    }
}

void fz_pdf_delete_page(fz_context *ctx, fz_document *doc, int number) {
    fz_try (ctx) {
        pdf_document *pdf = pdf_document_from_fz_document(ctx, doc);
        pdf_delete_page(ctx, pdf, number);
    }
    fz_catch (ctx) {
    }
}

int fz_pdf_insert_page(fz_context *ctx, fz_document *doc, fz_page *page, int after) {
    int ret = -1;
    fz_try (ctx) {
        pdf_document *pdf = pdf_document_from_fz_document(ctx, doc);
        pdf_page *pp = pdf_page_from_fz_page(ctx, page);
        pdf_obj *page_obj = pdf_keep_obj(ctx, pp->obj);
        pdf_insert_page(ctx, pdf, after, page_obj);
        pdf_drop_obj(ctx, page_obj);
        ret = 0;
    }
    fz_catch (ctx) {
        ret = -1;
    }
    return ret;
}

void fz_pdf_rotate_page(fz_context *ctx, fz_document *doc, int number, int rotation) {
    fz_try (ctx) {
        pdf_document *pdf = pdf_document_from_fz_document(ctx, doc);
        pdf_obj *page_obj = pdf_lookup_page_obj(ctx, pdf, number);
        int current_rotation = pdf_to_int(ctx, pdf_dict_get_inheritable(ctx, page_obj, PDF_NAME(Rotate)));
        pdf_dict_put_int(ctx, page_obj, PDF_NAME(Rotate), (current_rotation + rotation) % 360);
    }
    fz_catch (ctx) {
    }
}

char *fz_pdf_output_intent(fz_context *ctx, fz_document *doc) {
    char *ret = NULL;
    fz_try (ctx) {
        pdf_document *pdf = pdf_document_from_fz_document(ctx, doc);
        pdf_obj *catalog = pdf_trailer(ctx, pdf);
        catalog = pdf_dict_get(ctx, catalog, PDF_NAME(Root));
        pdf_obj *output_intents = pdf_dict_get(ctx, catalog, PDF_NAME(OutputIntents));
        if (pdf_is_array(ctx, output_intents) && pdf_array_len(ctx, output_intents) > 0) {
            pdf_obj *intent = pdf_array_get(ctx, output_intents, 0);
            pdf_obj *dest_output_profile = pdf_dict_get(ctx, intent, PDF_NAME(DestOutputProfile));
            if (pdf_is_indirect(ctx, dest_output_profile)) {
                ret = pdf_sprint_obj(ctx, NULL, 0, NULL, dest_output_profile, 0, 0);
            }
        }
    }
    fz_catch (ctx) {
        ret = NULL;
    }
    return ret;
}

void fz_save_document(fz_context *ctx, fz_document *doc, const char *filename, const fz_write_options *opts, const char *fmt) {
    fz_try (ctx) {
        pdf_document *pdf = pdf_document_from_fz_document(ctx, doc);
        pdf_write_options pdf_opts = pdf_default_write_options;
        if (opts != NULL) {
            pdf_opts.do_incremental = opts->incremental;
            pdf_opts.do_clean = opts->clean;
            pdf_opts.do_garbage = opts->garbage;
            pdf_opts.do_ascii = opts->ascii;
            pdf_opts.do_linear = opts->linear;
        }
        pdf_save_document(ctx, pdf, filename, &pdf_opts);
    }
    fz_catch (ctx) {
    }
}

/* Image-related wrappers */

int fz_count_page_images(fz_context *ctx, fz_page *page) {
    int ret = 0;
    fz_try (ctx) {
        pdf_page *pp = pdf_page_from_fz_page(ctx, page);
        pdf_obj *resources = pdf_dict_get(ctx, pp->obj, PDF_NAME(Resources));
        if (resources) {
            pdf_obj *xobj = pdf_dict_get(ctx, resources, PDF_NAME(XObject));
            if (xobj) {
                ret = pdf_dict_len(ctx, xobj);
            }
        }
    }
    fz_catch (ctx) {
        ret = 0;
    }
    return ret;
}

fz_image *fz_load_page_image(fz_context *ctx, fz_page *page, int index) {
    fz_image *ret = NULL;
    fz_try (ctx) {
        pdf_page *pp = pdf_page_from_fz_page(ctx, page);
        pdf_obj *resources = pdf_dict_get(ctx, pp->obj, PDF_NAME(Resources));
        if (resources) {
            pdf_obj *xobj = pdf_dict_get(ctx, resources, PDF_NAME(XObject));
            if (xobj && index < pdf_dict_len(ctx, xobj)) {
                pdf_obj *img_obj = pdf_dict_get_val(ctx, xobj, index);
                pdf_document *pdf = pdf_document_from_fz_document(ctx, page->doc);
                ret = pdf_load_image(ctx, pdf, img_obj);
            }
        }
    }
    fz_catch (ctx) {
        ret = NULL;
    }
    return ret;
}

int fz_image_width(fz_context *ctx, fz_image *image) {
    return image ? image->w : 0;
}

int fz_image_height(fz_context *ctx, fz_image *image) {
    return image ? image->h : 0;
}

/* Font-related wrappers */

int fz_count_page_fonts(fz_context *ctx, fz_page *page) {
    int ret = 0;
    fz_try (ctx) {
        pdf_page *pp = pdf_page_from_fz_page(ctx, page);
        pdf_obj *resources = pdf_dict_get(ctx, pp->obj, PDF_NAME(Resources));
        if (resources) {
            pdf_obj *fonts = pdf_dict_get(ctx, resources, PDF_NAME(Font));
            if (fonts) {
                ret = pdf_dict_len(ctx, fonts);
            }
        }
    }
    fz_catch (ctx) {
        ret = 0;
    }
    return ret;
}

/* Annotation wrappers */

fz_annot *fz_first_annot(fz_context *ctx, fz_page *page) {
    fz_annot *ret = NULL;
    fz_try (ctx) {
        pdf_page *pp = pdf_page_from_fz_page(ctx, page);
        ret = pdf_first_annot(ctx, pp);
    }
    fz_catch (ctx) {
        ret = NULL;
    }
    return ret;
}

fz_annot *fz_next_annot(fz_context *ctx, fz_annot *annot) {
    fz_annot *ret = NULL;
    fz_try (ctx) {
        ret = pdf_next_annot(ctx, annot);
    }
    fz_catch (ctx) {
        ret = NULL;
    }
    return ret;
}

char *fz_annot_contents(fz_context *ctx, fz_annot *annot) {
    char *ret = NULL;
    fz_try (ctx) {
        pdf_obj *obj = pdf_annot_obj(ctx, annot);
        pdf_obj *contents = pdf_dict_get(ctx, obj, PDF_NAME(Contents));
        if (contents) {
            ret = pdf_to_str_buf(ctx, contents);
        }
    }
    fz_catch (ctx) {
        ret = NULL;
    }
    return ret;
}

fz_rect fz_annot_rect(fz_context *ctx, fz_annot *annot) {
    fz_rect rect = fz_empty_rect;
    fz_try (ctx) {
        rect = pdf_bound_annot(ctx, annot);
    }
    fz_catch (ctx) {
        rect = fz_empty_rect;
    }
    return rect;
}

fz_annot *fz_create_annot(fz_context *ctx, fz_page *page, const char *type) {
    fz_annot *ret = NULL;
    fz_try (ctx) {
        pdf_page *pp = pdf_page_from_fz_page(ctx, page);
        enum pdf_annot_type annot_type = PDF_ANNOT_TEXT;
        if (type != NULL) {
            if (strcmp(type, "Highlight") == 0) annot_type = PDF_ANNOT_HIGHLIGHT;
            else if (strcmp(type, "Underline") == 0) annot_type = PDF_ANNOT_UNDERLINE;
            else if (strcmp(type, "StrikeOut") == 0) annot_type = PDF_ANNOT_STRIKE_OUT;
            else if (strcmp(type, "Square") == 0) annot_type = PDF_ANNOT_SQUARE;
            else if (strcmp(type, "Circle") == 0) annot_type = PDF_ANNOT_CIRCLE;
            else if (strcmp(type, "FreeText") == 0) annot_type = PDF_ANNOT_FREE_TEXT;
            else if (strcmp(type, "Text") == 0) annot_type = PDF_ANNOT_TEXT;
            else if (strcmp(type, "Line") == 0) annot_type = PDF_ANNOT_LINE;
            else if (strcmp(type, "PolyLine") == 0) annot_type = PDF_ANNOT_POLY_LINE;
            else if (strcmp(type, "Polygon") == 0) annot_type = PDF_ANNOT_POLYGON;
            else if (strcmp(type, "Ink") == 0) annot_type = PDF_ANNOT_INK;
        }
        ret = pdf_create_annot(ctx, pp, annot_type);
    }
    fz_catch (ctx) {
        ret = NULL;
    }
    return ret;
}

void fz_set_annot_contents(fz_context *ctx, fz_annot *annot, const char *contents) {
    fz_try (ctx) {
        pdf_obj *obj = pdf_annot_obj(ctx, annot);
        pdf_dict_put_text_string(ctx, obj, PDF_NAME(Contents), contents);
    }
    fz_catch (ctx) {
    }
}

void fz_set_annot_rect(fz_context *ctx, fz_annot *annot, fz_rect rect) {
    fz_try (ctx) {
        pdf_obj *obj = pdf_annot_obj(ctx, annot);
        pdf_dict_put_rect(ctx, obj, PDF_NAME(Rect), rect);
    }
    fz_catch (ctx) {
    }
}

void fz_drop_annot(fz_context *ctx, fz_annot *annot) {
    fz_try (ctx) {
        pdf_drop_annot(ctx, annot);
    }
    fz_catch (ctx) {
    }
}

void fz_apply_redactions(fz_context *ctx, fz_page *page, int flags) {
    fz_try (ctx) {
        pdf_page *pp = pdf_page_from_fz_page(ctx, page);
        pdf_document *pdf = pdf_document_from_fz_document(ctx, page->doc);
        pdf_redact_options opts;
        memset(&opts, 0, sizeof(opts));
        opts.image_method = (flags & 1) ? PDF_REDACT_IMAGE_REMOVE : PDF_REDACT_IMAGE_NONE;
        opts.text = (flags & 2) ? PDF_REDACT_TEXT_REMOVE : PDF_REDACT_TEXT_NONE;
        opts.line_art = PDF_REDACT_LINE_ART_NONE;
        pdf_redact_page(ctx, pdf, pp, &opts);
    }
    fz_catch (ctx) {
    }
}

void fz_remove_redactions(fz_context *ctx, fz_page *page) {
    fz_try (ctx) {
        pdf_page *pp = pdf_page_from_fz_page(ctx, page);
        fz_annot *annot = pdf_first_annot(ctx, pp);
        while (annot) {
            fz_annot *next = pdf_next_annot(ctx, annot);
            if (pdf_annot_type(ctx, annot) == PDF_ANNOT_REDACT) {
                pdf_delete_annot(ctx, pp, annot);
            }
            annot = next;
        }
    }
    fz_catch (ctx) {
    }
}

/* Search wrapper - note: fz_search_page already exists in MuPDF with different signature */

int fz_search_page_rects(fz_context *ctx, fz_page *page, const char *text, fz_rect *hits, int hit_count) {
    int ret = 0;
    fz_try (ctx) {
        fz_stext_page *text_page = fz_new_stext_page_from_page(ctx, page, NULL);
        fz_quad *quads = fz_malloc_array(ctx, hit_count, fz_quad);
        ret = fz_search_stext_page(ctx, text_page, text, NULL, quads, hit_count);
        for (int i = 0; i < ret; i++) {
            hits[i] = fz_rect_from_quad(quads[i]);
        }
        fz_free(ctx, quads);
        fz_drop_stext_page(ctx, text_page);
    }
    fz_catch (ctx) {
        ret = 0;
    }
    return ret;
}
