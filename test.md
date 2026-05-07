# md-view Test Document

This is a test document for the **md-view** terminal Markdown renderer.

## Features

### Text Formatting

- **Bold text** renders with bright/bold
- *Italic text* renders with italic/underline
- ~~Strikethrough~~ renders with crossed-out style
- `inline code` renders in cyan with backtick indicators
- [Links](https://example.com) show URL in dim

### Lists

Unordered:
- First item
  - Nested item
    - Deep nested
- Second item

Ordered:
1. First
2. Second
3. Third

### Task Lists

- [x] Completed task
- [ ] Pending task
- [x] Another done

## Code Blocks

```rust
fn main() {
    let message = "Hello, world!";
    println!("{}", message);
    for i in 0..10 {
        // loop body
        compute(i);
    }
}
```

```python
def greet(name: str) -> str:
    """Greet someone."""
    return f"Hello, {name}!"

class MyClass:
    def __init__(self):
        self.value = 42
```

## Tables

| Feature | Status | Notes |
|---------|--------|-------|
| Headers | Done | All levels |
| Bold | Done | Works |
| Code | Done | Syntax highlight |
| Tables | Done | Box-drawing |

## Blockquotes

> This is a blockquote.
> It can span multiple lines.
> And contains **formatted** text.

---

## Final Section

This is the end of the test document. The horizontal rule above should render as a full-width dim line.

Goodbye!
