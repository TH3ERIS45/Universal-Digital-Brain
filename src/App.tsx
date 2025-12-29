import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import "./App.css";

interface Resource {
  id: string;
  path?: string;
  title: string;
  type: 'note' | 'file' | 'link' | 'task';
  extra_metadata?: string;
}

interface NoteContent {
  id: string;
  title: string;
  content: string;
}

function App() {
  const [resources, setResources] = useState<Resource[]>([]);
  const [status, setStatus] = useState("");
  const [selectedResId, setSelectedResId] = useState<string | null>(null);

  // Editor State
  const [editorTitle, setEditorTitle] = useState("");
  const [editorContent, setEditorContent] = useState("");

  // Filter State
  const [activeTab, setActiveTab] = useState<'all' | 'note' | 'link' | 'task' | 'file'>('all');

  async function scanVault() {
    setStatus("Scanning...");
    try {
      const path = "/home/thyeris/Documents/NewProject/vault";
      const msg = await invoke("scan_vault", { path });
      setStatus(msg as string);
      loadResources();
    } catch (e) {
      setStatus("ErrorStr: " + e);
      console.error(e);
    }
  }

  async function loadResources() {
    try {
      const result = await invoke("get_all_resources");
      console.log(result);
      setResources(result as Resource[]);
    } catch (e) {
      setStatus("Error loading resources: " + e);
    }
  }

  async function handleCreate(type: 'note' | 'link' | 'task') {
    try {
      let id;
      if (type === 'note') {
        const title = "New Note";
        id = await invoke("create_note", { title, content: "" });
      } else if (type === 'link') {
        const title = prompt("Enter Link Title:");
        if (!title) return;
        const url = prompt("Enter URL:");
        if (!url) return;
        id = await invoke("create_link", { title, url });
      } else if (type === 'task') {
        const title = prompt("Enter Task:");
        if (!title) return;
        id = await invoke("create_task", { title });
      }
      await loadResources();
      if (id && type === 'note') handleSelect(id as string, 'note');
    } catch (e) {
      setStatus("Error creating: " + e);
    }
  }

  async function handleSelect(id: string, type: string) {
    if (type === 'note') {
      try {
        const content = await invoke("get_note_content", { id }) as NoteContent;
        setSelectedResId(id);
        setEditorTitle(content.title);
        setEditorContent(content.content);
      } catch (e) {
        setStatus("Error opening note: " + e);
      }
    } else {
      setSelectedResId(null);
      setStatus(`Selected ${type}: ${id}`);
    }
  }

  async function handleSaveNote() {
    if (!selectedResId) return;
    try {
      await invoke("update_note", {
        id: selectedResId,
        title: editorTitle,
        content: editorContent
      });
      setStatus("Saved!");
      loadResources();
    } catch (e) {
      setStatus("Error saving: " + e);
    }
  }

  async function handleDelete() {
    if (!selectedResId) return;
    if (!confirm("Delete this?")) return;
    try {
      await invoke("delete_note", { id: selectedResId });
      setSelectedResId(null);
      loadResources();
    } catch (e) {
      setStatus("Error deleting: " + e);
    }
  }

  const filteredResources = resources.filter(r => activeTab === 'all' || r.type === activeTab);

  useEffect(() => {
    scanVault();
  }, []);

  return (
    <div className="app-container">
      <nav className="sidebar glass-panel">
        <h3>Universal Brain</h3>

        {/* Resource Type Filters */}
        <div style={{ display: 'flex', gap: '5px', marginBottom: '10px', fontSize: '0.8em' }}>
          <span onClick={() => setActiveTab('all')} style={{ cursor: 'pointer', opacity: activeTab === 'all' ? 1 : 0.5 }}>All</span>
          <span onClick={() => setActiveTab('note')} style={{ cursor: 'pointer', opacity: activeTab === 'note' ? 1 : 0.5 }}>Notes</span>
          <span onClick={() => setActiveTab('link')} style={{ cursor: 'pointer', opacity: activeTab === 'link' ? 1 : 0.5 }}>Links</span>
          <span onClick={() => setActiveTab('task')} style={{ cursor: 'pointer', opacity: activeTab === 'task' ? 1 : 0.5 }}>Tasks</span>
        </div>

        <div style={{ display: 'flex', gap: '5px', marginBottom: '10px' }}>
          <select onChange={(e) => handleCreate(e.target.value as any)} style={{ flex: 1, background: 'var(--accent-primary)', border: 'none', color: 'white', padding: '5px', borderRadius: '4px' }}>
            <option value="">+ New...</option>
            <option value="note">Note</option>
            <option value="link">Link</option>
            <option value="task">Task</option>
          </select>
        </div>

        <div className="note-list" style={{ overflowY: 'auto', flex: 1 }}>
          {filteredResources.map(res => (
            <div key={res.id}
              className={`note-item ${selectedResId === res.id ? 'active' : ''}`}
              onClick={() => handleSelect(res.id, res.type)}
              style={{
                padding: '8px',
                cursor: 'pointer',
                color: selectedResId === res.id ? 'white' : 'var(--text-main)',
                background: selectedResId === res.id ? 'var(--accent-primary)' : 'transparent',
                borderRadius: '4px',
                marginBottom: '2px',
                display: 'flex',
                alignItems: 'center'
              }}>
              <span style={{ marginRight: '8px', fontSize: '1.2em' }}>
                {res.type === 'note' ? '📄' : res.type === 'link' ? '🔗' : res.type === 'task' ? '☑️' : '📁'}
              </span>
              <div style={{ overflow: 'hidden', textOverflow: 'ellipsis', whiteSpace: 'nowrap' }}>
                {res.title}
                {res.extra_metadata && res.type === 'task' && (
                  <span style={{ fontSize: '0.7em', marginLeft: '5px', opacity: 0.7 }}>
                    {JSON.parse(res.extra_metadata).status}
                  </span>
                )}
              </div>
            </div>
          ))}
        </div>
      </nav>

      <main className="main-content">
        {selectedResId ? (
          <div className="editor-container" style={{ display: 'flex', flexDirection: 'column', height: '100%', padding: '20px' }}>
            <div style={{ display: 'flex', gap: '10px', marginBottom: '10px', alignItems: 'center' }}>
              <input
                value={editorTitle}
                onChange={(e) => setEditorTitle(e.target.value)}
                style={{ fontSize: '1.5em', fontWeight: 'bold', background: 'transparent', border: 'none', borderBottom: '1px solid var(--border-subtle)', flex: 1 }}
              />
              <button onClick={handleSaveNote}>Save</button>
              <button onClick={handleDelete} style={{ background: 'red' }}>Delete</button>
            </div>
            <textarea
              value={editorContent}
              onChange={(e) => setEditorContent(e.target.value)}
              style={{
                flex: 1,
                background: 'transparent',
                color: 'var(--text-main)',
                border: 'none',
                resize: 'none',
                fontSize: '1.1em',
                lineHeight: '1.6',
                outline: 'none',
                fontFamily: 'monospace'
              }}
            />
          </div>
        ) : (
          <div style={{ padding: '40px', maxWidth: '800px', margin: '0 auto' }}>
            <div className="glass" style={{ padding: '20px', borderRadius: '12px', marginBottom: '20px' }}>
              <h2>Welcome to your Digital Brain</h2>
              <p style={{ marginTop: '10px', color: 'var(--text-muted)' }}>
                {resources.length === 0 ? "Empty Vault. Create something!" : "Select an item to view."}
              </p>
            </div>

            {/* Dashboard / Summary View */}
            <div style={{ display: 'grid', gridTemplateColumns: '1fr 1fr', gap: '20px' }}>
              <div className="glass" style={{ padding: '15px' }}>
                <h4>Recent Tasks</h4>
                {resources.filter(r => r.type === 'task').slice(0, 5).map(t => (
                  <div key={t.id} style={{ padding: '5px 0', borderBottom: '1px solid var(--border-subtle)' }}>
                    ☑️ {t.title}
                  </div>
                ))}
              </div>
              <div className="glass" style={{ padding: '15px' }}>
                <h4>Quick Links</h4>
                {resources.filter(r => r.type === 'link').slice(0, 5).map(l => (
                  <div key={l.id} style={{ padding: '5px 0', borderBottom: '1px solid var(--border-subtle)' }}>
                    🔗 <a href={l.extra_metadata ? JSON.parse(l.extra_metadata).url : '#'} target="_blank" style={{ color: 'var(--text-accent)' }}>{l.title}</a>
                  </div>
                ))}
              </div>
            </div>

            <div style={{ marginTop: '20px', color: 'var(--text-accent)' }}>
              Status: {status}
            </div>
          </div>
        )}
      </main>

      <aside className="right-panel">
        <h4>Analysis</h4>
        <div style={{ marginTop: '20px', fontSize: '0.9em', color: 'var(--text-muted)' }}>
          <p>Notes: {resources.filter(r => r.type === 'note').length}</p>
          <p>Files: {resources.filter(r => r.type === 'file').length}</p>
          <p>Links: {resources.filter(r => r.type === 'link').length}</p>
          <p>Tasks: {resources.filter(r => r.type === 'task').length}</p>
        </div>
      </aside>
    </div>
  );
}

export default App;
