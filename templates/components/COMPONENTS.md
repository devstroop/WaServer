# WAS Design System Components

> WhatsApp-inspired component library for the WhatsApp Server (WAS) project.

## Quick Start

Include all components in your template:
```jinja2
{% include "components/_all.html" %}
```

Or include individual components:
```jinja2
{% include "components/button.html" %}
{% include "components/input.html" %}
```

---

## Component Index

### Foundation
| Component | File | Description |
|-----------|------|-------------|
| **Design Tokens** | `_tokens.html` | CSS variables for colors, spacing, typography, shadows, z-index |
| **All Components** | `_all.html` | Includes all components at once |

### Core Components

| Component | File | Status | Description |
|-----------|------|--------|-------------|
| **Button** | `button.html` | ✅ Complete | Buttons with WhatsApp pill style, variants, sizes, icon buttons |
| **Input** | `input.html` | ✅ Complete | Text inputs, textareas, selects, checkboxes, radios, switches |
| **Card** | `card.html` | ✅ Complete | Cards, stat cards, action cards |
| **Badge** | `badge.html` | ✅ Complete | Status badges, notification counts, tags |
| **Avatar** | `avatar.html` | ✅ Complete | User avatars with fallbacks, status indicators, groups |
| **Alert** | `alert.html` | ✅ Complete | Inline alert banners (success, warning, danger, info) |
| **Code** | `code.html` | ✅ Complete | Inline code, code blocks with copy button |
| **Modal** | `modal.html` | ✅ Complete | Modal dialogs, alert dialogs, sheets |
| **Dropdown** | `dropdown.html` | ✅ Complete | Dropdown menus, context menus, popovers |
| **Toast** | `toast.html` | ✅ Complete | Toast notifications with JS API |
| **Table** | `table.html` | ✅ Complete | Data tables, pagination, empty states |
| **Skeleton** | `skeleton.html` | ✅ Complete | Loading skeletons, spinners, progress bars |
| **Chat** | `chat.html` | ✅ Complete | Chat list items, message bubbles, typing indicator |

### Planned Components

| Component | Priority | Description |
|-----------|----------|-------------|
| **Tabs** | 🟡 Medium | Tab navigation for settings pages |
| **Empty State** | 🟡 Medium | Standardized empty state patterns |
| **Separator** | 🟢 Low | Visual dividers (horizontal rules, with labels) |
| **Tooltip** | 🟢 Low | Hover tooltips (may extend dropdown.html) |
| **Progress** | 🟢 Low | Progress bars and step indicators |

---

## Design Tokens

Located in `_tokens.html`. Key variables:

### Colors
```css
/* Brand */
--color-brand: #00a884;           /* WhatsApp teal */
--color-brand-hover: #008f72;
--color-accent: #008069;          /* Header teal */

/* Backgrounds */
--color-background: #ffffff;
--color-background-subtle: #f0f2f5;
--color-background-chat: #efeae2;
--color-background-panel: #ffffff;

/* Message Bubbles */
--color-bubble-outgoing: #d9fdd3; /* Light green */
--color-bubble-incoming: #ffffff;

/* Text */
--color-foreground: #111b21;
--color-foreground-muted: #667781;
--color-foreground-subtle: #8696a0;

/* Semantic */
--color-success: #00a884;
--color-danger: #ea0038;
--color-warning: #ffc107;
--color-info: #53bdeb;
```

### Spacing (4px grid)
```css
--space-1: 0.25rem;   /* 4px */
--space-2: 0.5rem;    /* 8px */
--space-3: 0.75rem;   /* 12px */
--space-4: 1rem;      /* 16px */
--space-6: 1.5rem;    /* 24px */
--space-8: 2rem;      /* 32px */
```

### Border Radius
```css
--radius-sm: 0.25rem;    /* 4px - buttons, small elements */
--radius-md: 0.375rem;   /* 6px - cards, inputs */
--radius-lg: 0.5rem;     /* 8px - larger containers */
--radius-full: 9999px;   /* Pills, circles */
```

---

## Component Usage

### Button

```html
<!-- Primary (WhatsApp green) -->
<button class="btn btn-default">Send Message</button>

<!-- Secondary -->
<button class="btn btn-secondary">Cancel</button>

<!-- Ghost (icon button style) -->
<button class="btn btn-ghost">More</button>

<!-- Destructive -->
<button class="btn btn-destructive">Delete</button>

<!-- Icon button -->
<button class="btn btn-icon btn-ghost">
    <i class="bi bi-three-dots-vertical"></i>
</button>

<!-- Sizes -->
<button class="btn btn-default btn-sm">Small</button>
<button class="btn btn-default btn-lg">Large</button>
```

### Input

```html
<!-- Basic input -->
<input type="text" class="input" placeholder="Enter text...">

<!-- With error state -->
<input type="text" class="input input-error">

<!-- Search bar (WhatsApp pill style) -->
<div class="search-bar">
    <i class="bi bi-search search-icon"></i>
    <input type="text" class="input" placeholder="Search...">
</div>

<!-- Form field with label -->
<div class="form-field">
    <label class="form-label">Email</label>
    <input type="email" class="input">
    <span class="form-helper">We'll never share your email</span>
</div>

<!-- Switch toggle -->
<label class="switch-wrapper">
    <input type="checkbox" class="switch">
    <span class="switch-label">Enable notifications</span>
</label>
```

### Card

```html
<!-- Basic card -->
<div class="card">
    <div class="card-header">
        <h3 class="card-title">Title</h3>
    </div>
    <div class="card-content">
        Content here
    </div>
</div>

<!-- Stat card (dashboard) -->
<div class="card stat-card">
    <div class="stat-header">
        <span class="stat-title">Server Status</span>
        <i class="bi bi-server stat-icon"></i>
    </div>
    <div class="stat-value stat-value-success">Healthy</div>
    <div class="stat-subtitle">Version 1.0.0</div>
</div>

<!-- Action card -->
<div class="card action-card card-interactive">
    <div class="action-header">
        <div class="action-icon"><i class="bi bi-chat-dots"></i></div>
        <h4 class="action-title">Start Messaging</h4>
    </div>
    <p class="action-description">Send messages through the chat interface</p>
    <button class="btn btn-default">Open Chats</button>
</div>
```

### Badge

```html
<!-- Default (brand) -->
<span class="badge">New</span>

<!-- Variants -->
<span class="badge badge-success">Active</span>
<span class="badge badge-danger">Error</span>
<span class="badge badge-warning">Pending</span>

<!-- Muted variants -->
<span class="badge badge-success-muted">Connected</span>

<!-- Status indicator -->
<span class="status-indicator status-online">
    <span class="status-dot"></span>
    Online
</span>

<!-- Unread count (WhatsApp style) -->
<span class="badge">3</span>
```

### Alert

```html
<!-- Info (default) -->
<div class="alert alert-info">
    <i class="bi bi-info-circle alert-icon"></i>
    <div class="alert-content">
        <h4 class="alert-title">Information</h4>
        <p class="alert-description">This is an informational message.</p>
    </div>
</div>

<!-- Success -->
<div class="alert alert-success">
    <i class="bi bi-check-circle alert-icon"></i>
    <p class="alert-description">Operation completed successfully.</p>
</div>

<!-- Warning -->
<div class="alert alert-warning">
    <i class="bi bi-exclamation-triangle alert-icon"></i>
    <p class="alert-description">Please review before continuing.</p>
</div>

<!-- Danger -->
<div class="alert alert-danger">
    <i class="bi bi-x-circle alert-icon"></i>
    <p class="alert-description">An error occurred.</p>
</div>

<!-- Compact (inline style) -->
<div class="alert alert-warning alert-compact">
    <i class="bi bi-exclamation-triangle alert-icon"></i>
    <p class="alert-description"><strong>Important:</strong> Copy your token now.</p>
</div>

<!-- WhatsApp-style info message -->
<div class="info-message">
    <i class="bi bi-lock-fill info-message-icon"></i>
    Messages are end-to-end encrypted
</div>
```

### Code

```html
<!-- Inline code -->
<code class="code">Bearer token</code>

<!-- Code block with header and copy button -->
<div class="code-block">
    <div class="code-header">
        <span class="code-lang">Python</span>
        <button class="code-copy" onclick="copyCode(this)">
            <i class="bi bi-clipboard"></i>
        </button>
    </div>
    <pre class="code-pre"><code>def hello():
    print("Hello, World!")</code></pre>
</div>

<!-- Simple code block (no header) -->
<div class="code-block code-block-simple">
    <pre class="code-pre"><code>npm install was-client</code></pre>
</div>

<!-- Copyable inline code -->
<div class="code-copyable">
    <span class="code-copyable-text">was_abc123xyz</span>
    <button class="code-copy" onclick="copyCopyable(this)">
        <i class="bi bi-clipboard"></i>
    </button>
</div>
```

### Avatar

```html
<!-- Basic avatar -->
<div class="avatar">
    <img src="/photo.jpg" class="avatar-image" alt="User">
    <span class="avatar-fallback">JD</span>
</div>

<!-- Sizes -->
<div class="avatar avatar-sm">...</div>
<div class="avatar avatar-lg">...</div>

<!-- With status -->
<div class="avatar avatar-with-status">
    <img src="/photo.jpg" class="avatar-image">
    <span class="avatar-status avatar-status-online"></span>
</div>

<!-- Avatar group -->
<div class="avatar-group">
    <div class="avatar avatar-sm">...</div>
    <div class="avatar avatar-sm">...</div>
    <div class="avatar avatar-sm avatar-count">+3</div>
</div>
```

### Chat Components

```html
<!-- Chat list item -->
<div class="chat-list-item unread" onclick="selectChat('123')">
    <div class="avatar chat-avatar">
        <span class="avatar-fallback">JD</span>
    </div>
    <div class="chat-content">
        <div class="chat-header">
            <span class="chat-name u-trim">John Doe</span>
            <span class="chat-time">12:30</span>
        </div>
        <div class="chat-preview">
            <span class="chat-preview-text u-trim">Hey, how are you?</span>
            <span class="unread-badge">3</span>
        </div>
    </div>
</div>

<!-- Message bubble (outgoing) -->
<div class="message-container outgoing">
    <div class="message-bubble outgoing">
        <p class="message-text">Hello! How are you?</p>
        <div class="message-footer">
            <span class="message-time">12:30</span>
            <span class="message-status read">
                <span class="check-double"></span>
            </span>
        </div>
    </div>
</div>

<!-- Message bubble (incoming) -->
<div class="message-container incoming">
    <div class="message-bubble incoming">
        <p class="message-text">I'm doing great, thanks!</p>
        <div class="message-footer">
            <span class="message-time">12:31</span>
        </div>
    </div>
</div>

<!-- Typing indicator -->
<div class="typing-indicator">
    <span class="typing-dot"></span>
    <span class="typing-dot"></span>
    <span class="typing-dot"></span>
</div>

<!-- System message -->
<div class="message-system">
    <span class="message-system-text">
        Messages are end-to-end encrypted
    </span>
</div>
```

### Modal

```html
<!-- Trigger -->
<button onclick="toggleModal('my-modal')">Open Modal</button>

<!-- Modal -->
<div class="modal-overlay" data-modal="my-modal">
    <div class="modal">
        <div class="modal-header">
            <h2 class="modal-title">Confirm Action</h2>
            <button class="modal-close" onclick="toggleModal('my-modal')">
                <i class="bi bi-x"></i>
            </button>
        </div>
        <div class="modal-content">
            Are you sure you want to continue?
        </div>
        <div class="modal-footer">
            <button class="btn btn-secondary">Cancel</button>
            <button class="btn btn-default">Confirm</button>
        </div>
    </div>
</div>
```

### Toast

```javascript
// Using the JS API
toast.success('Message sent successfully');
toast.error('Failed to send message');
toast.warning('Connection unstable');
toast.info('New message received');
toast.loading('Sending...');

// With options
toast.show('Custom message', {
    variant: 'success',
    duration: 5000,
    title: 'Success!'
});
```

### Dropdown

```html
<div class="dropdown">
    <button class="btn btn-icon btn-ghost" data-dropdown-trigger>
        <i class="bi bi-three-dots-vertical"></i>
    </button>
    <div class="dropdown-content dropdown-right">
        <button class="dropdown-item">
            <i class="bi bi-pencil"></i>
            Edit
        </button>
        <button class="dropdown-item">
            <i class="bi bi-copy"></i>
            Duplicate
        </button>
        <div class="dropdown-separator"></div>
        <button class="dropdown-item dropdown-item-danger">
            <i class="bi bi-trash"></i>
            Delete
        </button>
    </div>
</div>
```

### Table

```html
<div class="table-container">
    <table class="table">
        <thead>
            <tr>
                <th class="th-sortable">Name <i class="bi bi-chevron-expand"></i></th>
                <th>Status</th>
                <th>Created</th>
                <th></th>
            </tr>
        </thead>
        <tbody>
            <tr>
                <td>API Token</td>
                <td><span class="badge badge-success">Active</span></td>
                <td>2024-01-15</td>
                <td>
                    <button class="btn btn-icon btn-ghost btn-sm">
                        <i class="bi bi-trash"></i>
                    </button>
                </td>
            </tr>
        </tbody>
    </table>
</div>

<!-- Pagination -->
<div class="pagination">
    <button class="pagination-btn" disabled>Previous</button>
    <span class="pagination-info">Page 1 of 5</span>
    <button class="pagination-btn">Next</button>
</div>
```

### Skeleton Loading

```html
<!-- Text skeleton -->
<div class="skeleton skeleton-text"></div>
<div class="skeleton skeleton-text" style="width: 60%"></div>

<!-- Avatar skeleton -->
<div class="skeleton skeleton-circle" style="width: 48px; height: 48px"></div>

<!-- Card skeleton -->
<div class="skeleton skeleton-rect" style="height: 120px"></div>

<!-- Spinner -->
<div class="spinner"></div>

<!-- Pulse loader -->
<div class="pulse-loader">
    <span></span>
    <span></span>
    <span></span>
</div>
```

---

## Utilities

### Text Trimming
```html
<span class="u-trim">This text will be truncated with ellipsis...</span>
```

### Custom Scrollbar
```html
<div class="u-scroll" style="height: 300px">
    <!-- Scrollable content with thin WhatsApp-style scrollbar -->
</div>
```

---

## Dark Mode

The design system automatically supports dark mode via `data-bs-theme="dark"` on the `<html>` element.

Toggle theme:
```javascript
function toggleTheme() {
    const html = document.documentElement;
    const current = html.getAttribute('data-bs-theme');
    html.setAttribute('data-bs-theme', current === 'dark' ? 'light' : 'dark');
}
```

---

## File Structure

```
templates/components/
├── _all.html           # Include all components
├── _tokens.html        # Design tokens (CSS variables + utilities)
├── alert.html          # Alert/banner component
├── avatar.html         # Avatar component
├── badge.html          # Badge component
├── button.html         # Button component
├── card.html           # Card component
├── chat.html           # Chat-specific components
├── code.html           # Code/syntax component
├── dropdown.html       # Dropdown/menu component
├── input.html          # Form input components
├── modal.html          # Modal dialog component
├── skeleton.html       # Loading skeleton component
├── table.html          # Table component
├── toast.html          # Toast notification component
└── COMPONENTS.md       # This documentation
```

---

## Migration Guide

### From Bootstrap classes

| Bootstrap | Design System |
|-----------|---------------|
| `btn btn-primary` | `btn btn-default` |
| `btn btn-success` | `btn btn-default` (uses brand color) |
| `btn btn-outline-secondary` | `btn btn-outline` |
| `form-control` | `input` |
| `form-check` | `checkbox-wrapper` / `switch-wrapper` |
| `card` | `card` (enhanced) |
| `badge bg-success` | `badge badge-success` |
| `spinner-border` | `spinner` |

---

## Contributing

When adding new components:

1. Create a new `.html` file in `templates/components/`
2. Add comprehensive Jinja2 comments with usage examples
3. Use design tokens for all colors, spacing, and typography
4. Support both light and dark themes
5. Add the component to `_all.html`
6. Document the component in this file

---

*Last updated: February 2026*
