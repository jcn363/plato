# Navigation

## Manga Mode

Plato supports right-to-left reading for manga and other content that uses Japanese reading direction.

### Enabling Manga Mode

1. Open the reader menu (tap the center of the screen)
2. Select **Settings** → **Reading**
3. Toggle **Manga Mode** to **On**

### Navigation in Manga Mode

When manga mode is enabled, navigation directions are reversed:

| Action              | Normal Mode       | Manga Mode        |
|---------------------|-------------------|-------------------|
| Swipe left (West)   | Next page         | Previous page     |
| Swipe right (East)  | Previous page     | Next page         |
| Bottom bar ← button | Previous page     | Next page         |
| Bottom bar → button | Next page         | Previous page     |
| Arrow gesture West  | Previous chapter  | Next chapter      |
| Arrow gesture East  | Next chapter      | Previous chapter  |
| Corner NW           | Previous bookmark | Next bookmark     |
| Corner NE           | Next bookmark     | Previous bookmark |

This allows for intuitive right-to-left reading, where swiping from right to left advances the page, matching traditional manga reading direction.

## Naming Pages

To name a page, hold the current page indicator and select the *Name* entry. A page name can be an uppercase ASCII letter, a lowercase roman numeral or an arabic numeral.

Once a page is named, you can jump to any page above it in the same category. For example if you've defined page 15 as *vi*, by entering *'ix*, in the *Go to page* input field, you'll jump to page 18.

You can also select a page name in the book's text and jump to it by tapping *Go To* in the selection menu. This can be particularly useful within a book's index.

## Overriding the TOC

You can override a book's TOC by adding a *toc* key to the corresponding entry in `.metadata.json`:

```text
{
 ⋮
 "toc": [
  ["Chapter 1", 17],
  ["Chapter 2", 46],
  ["Chapter 3", 88],
  ⋮
 ],
 ⋮
},
```

Page names can also be used instead of page numbers:

```text
{
 ⋮
 "toc": [
  ["Preface", "'viii"],
  ["Acknowledgments", "'xvii"],
  ["Introduction", "'1"],
  ["Section 1", "'16", [["Chapter 1", "'16"],
         ["Chapter 2", "'47"],
         ["Chapter 3", "'62"]]],
  ⋮
  ["Conclusion", "'141"],
  ["Notes", "'145"],
  ["Index", "'169"]
 ],
 ⋮
},
```

For the page names to be resolved, you'll need to name the first page of each category.

## Special Notations

`-` or `+` can be prepended to a page number to jump to a relative page.

Instead of the page number, you can specify one of the following characters:

- `(` and `)` to jump to the first and last page.
- `_` to jump to a random page.

If a number ending with `%` is given, it will be interpreted as a percentage of the book's page count.
