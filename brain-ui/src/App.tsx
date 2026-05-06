import { useState, useEffect, useRef, useMemo, Suspense } from 'react';
import { Canvas, useFrame } from '@react-three/fiber';
import { Text, OrbitControls, Float, Stars } from '@react-three/drei';
import { EffectComposer, Bloom, Scanline, Noise } from '@react-three/postprocessing';
import * as THREE from 'three';
import axios from 'axios';

interface KnowledgeNode {
  title: string;
  content: string;
  source_category: string;
  reliability_score: number;
  location?: string;
  language: string;
}

interface Source {
  url: string;
  category: string;
  enabled: boolean;
  failure_count: number;
  last_error?: string;
}

const API_BASE = window.location.hostname === 'localhost' ? 'http://localhost:4000' : `https://api.${window.location.host}`;

const Node = ({ node, position }: { node: KnowledgeNode; position: [number, number, number] }) => {
  const [hovered, setHovered] = useState(false);
  const color = useMemo(() => {
    if (node.source_category === 'Personal') return '#00ffaa';
    if (node.source_category === 'Local_News') return '#aaff00';
    return '#00ff44';
  }, [node.source_category]);

  return (
    <Float speed={2} rotationIntensity={0.5} floatIntensity={0.5}>
      <mesh
        position={position}
        onPointerOver={(e) => { e.stopPropagation(); setHovered(true); }}
        onPointerOut={() => setHovered(false)}
      >
        <boxGeometry args={[0.3, 0.3, 0.3]} />
        <meshStandardMaterial
          color={color}
          emissive={color}
          emissiveIntensity={hovered ? 10 : 2}
        />
        {hovered && (
          <group position={[0, 0.6, 0]}>
            <Text
              fontSize={0.25}
              color="#00ff44"
              anchorX="center"
              anchorY="middle"
            >
              {node.title.substring(0, 50)}{node.title.length > 50 ? '...' : ''}
            </Text>
            {node.location && (
              <Text
                position={[0, -0.3, 0]}
                fontSize={0.15}
                color="#aaff00"
                anchorX="center"
                anchorY="middle"
              >
                LOC: {node.location} [{node.language.toUpperCase()}]
              </Text>
            )}
          </group>
        )}
      </mesh>
    </Float>
  );
};

const DataStreams = () => {
  const count = 200;
  const lines = useMemo(() => {
    return Array.from({ length: count }, () => ({
      x: (Math.random() - 0.5) * 30,
      z: (Math.random() - 0.5) * 30,
      y: Math.random() * 20,
      speed: 0.05 + Math.random() * 0.1,
    }));
  }, []);

  const refs = useRef<(THREE.Mesh | null)[]>([]);
  useFrame(() => {
    lines.forEach((line, i) => {
      if (refs.current[i]) {
        refs.current[i]!.position.y -= line.speed;
        if (refs.current[i]!.position.y < -10) {
          refs.current[i]!.position.y = 10;
        }
      }
    });
  });

  return (
    <>
      {lines.map((line, i) => (
        <mesh key={i} ref={(el) => (refs.current[i] = el)} position={[line.x, line.y, line.z]}>
          <boxGeometry args={[0.02, 1, 0.02]} />
          <meshBasicMaterial color="#00ff44" transparent opacity={0.3} />
        </mesh>
      ))}
    </>
  );
};

const SettingsModal = ({ isOpen, onClose }: { isOpen: boolean; onClose: () => void }) => {
  const [activeTab, setActiveTab] = useState<'sources' | 'status' | 'credentials'>('sources');
  const [sources, setSources] = useState<Source[]>([]);
  const [status, setStatus] = useState<any>(null);
  const [newUrl, setNewUrl] = useState('');
  const [newCategory, setNewCategory] = useState('Global_News');

  const fetchData = async () => {
    try {
      if (activeTab === 'sources') {
        const res = await axios.get(`${API_BASE}/sources`);
        setSources(res.data);
      } else if (activeTab === 'status') {
        const res = await axios.get(`${API_BASE}/status`);
        setStatus(res.data);
      }
    } catch (err) {
      console.error('Failed to fetch settings data', err);
    }
  };

  useEffect(() => {
    if (isOpen) fetchData();
  }, [isOpen, activeTab]);

  const addSource = async () => {
    if (!newUrl) return;
    try {
      await axios.post(`${API_BASE}/sources`, { url: newUrl, category: newCategory });
      setNewUrl('');
      fetchData();
    } catch (err) {
      alert('Failed to add source');
    }
  };

  if (!isOpen) return null;

  return (
    <div style={{
      position: 'fixed',
      top: 0, left: 0, right: 0, bottom: 0,
      backgroundColor: 'rgba(0,0,0,0.9)',
      zIndex: 100,
      display: 'flex',
      alignItems: 'center',
      justifyContent: 'center',
      color: '#00ff44',
      fontFamily: 'monospace'
    }}>
      <div style={{
        backgroundColor: '#050505',
        border: '1px solid #00ff44',
        padding: '30px',
        width: '90%',
        maxWidth: '900px',
        height: '80vh',
        display: 'flex',
        flexDirection: 'column',
        boxShadow: '0 0 30px rgba(0,255,68,0.2)'
      }}>
        <div style={{ display: 'flex', justifyContent: 'space-between', marginBottom: '30px' }}>
          <h2 style={{ margin: 0, letterSpacing: '2px' }}>// SYSTEM_CONFIGURATION_INTERFACE</h2>
          <button onClick={onClose} style={{
            background: 'transparent', border: '1px solid #00ff44', color: '#00ff44', cursor: 'pointer', padding: '5px 15px'
          }}>[ DISCONNECT ]</button>
        </div>

        <div style={{ display: 'flex', gap: '20px', marginBottom: '20px' }}>
          {['sources', 'status', 'credentials'].map((tab: any) => (
            <button
              key={tab}
              onClick={() => setActiveTab(tab)}
              style={{
                background: activeTab === tab ? '#00ff44' : 'transparent',
                color: activeTab === tab ? '#000' : '#00ff44',
                border: '1px solid #00ff44',
                padding: '10px 20px',
                cursor: 'pointer',
                textTransform: 'uppercase'
              }}
            >
              {tab}
            </button>
          ))}
        </div>

        <div style={{ flex: 1, overflowY: 'auto', border: '1px solid #222', padding: '20px' }}>
          {activeTab === 'sources' && (
            <>
              <div style={{ marginBottom: '30px', padding: '15px', background: 'rgba(0,255,68,0.05)' }}>
                <h3 style={{ marginTop: 0 }}>Add New Data Stream</h3>
                <div style={{ display: 'flex', gap: '10px' }}>
                  <input
                    placeholder="RSS / ATOM URL"
                    value={newUrl}
                    onChange={(e) => setNewUrl(e.target.value)}
                    style={{ flex: 1, background: '#000', color: '#00ff44', border: '1px solid #00ff44', padding: '10px' }}
                  />
                  <select
                    value={newCategory}
                    onChange={(e) => setNewCategory(e.target.value)}
                    style={{ background: '#000', color: '#00ff44', border: '1px solid #00ff44', padding: '10px' }}
                  >
                    <option value="Global_News">Global_News</option>
                    <option value="Local_News">Local_News</option>
                    <option value="Personal">Personal</option>
                  </select>
                  <button onClick={addSource} style={{ background: '#00ff44', color: '#000', border: 'none', padding: '0 25px', cursor: 'pointer', fontWeight: 'bold' }}>INITIALIZE</button>
                </div>
              </div>

              <h3>Active Intelligence Streams ({sources.length})</h3>
              <table style={{ width: '100%', borderCollapse: 'collapse' }}>
                <thead>
                  <tr style={{ borderBottom: '1px solid #00ff44', textAlign: 'left' }}>
                    <th style={{ padding: '10px' }}>LINK</th>
                    <th style={{ padding: '10px' }}>URL</th>
                    <th style={{ padding: '10px' }}>HEALTH</th>
                    <th style={{ padding: '10px' }}>TERMINATE</th>
                  </tr>
                </thead>
                <tbody>
                  {sources.map(s => (
                    <tr key={s.url} style={{ borderBottom: '1px solid #111' }}>
                      <td style={{ padding: '10px' }}>
                        <input
                          type="checkbox"
                          checked={s.enabled}
                          onChange={async (e) => {
                            await axios.put(`${API_BASE}/sources/${encodeURIComponent(s.url)}`, { enabled: e.target.checked });
                            fetchData();
                          }}
                        />
                      </td>
                      <td style={{ padding: '10px', fontSize: '0.75rem', maxWidth: '350px', overflow: 'hidden', textOverflow: 'ellipsis' }}>{s.url}</td>
                      <td style={{ padding: '10px', color: s.failure_count > 0 ? '#ff4444' : '#00ff44' }}>{s.failure_count === 0 ? 'OPTIMAL' : `FAILED_${s.failure_count}`}</td>
                      <td style={{ padding: '10px' }}>
                        <button onClick={async () => {
                          await axios.delete(`${API_BASE}/sources/${encodeURIComponent(s.url)}`);
                          fetchData();
                        }} style={{ background: 'transparent', border: 'none', color: '#ff4444', cursor: 'pointer' }}>[X]</button>
                      </td>
                    </tr>
                  ))}
                </tbody>
              </table>
            </>
          )}

          {activeTab === 'status' && status && (
            <div style={{ fontSize: '1.2rem', lineHeight: '2' }}>
              <p>// CORE_STATUS: <span style={{ color: '#fff' }}>{status.status.toUpperCase()}</span></p>
              <p>// VERSION: <span style={{ color: '#fff' }}>{status.version}</span></p>
              <p>// WEAVIATE_CONNECTION: <span style={{ color: status.systems.weaviate === 'connected' ? '#00ff44' : '#ff4444' }}>{status.systems.weaviate.toUpperCase()}</span></p>
              <p>// REDIS_CACHE: <span style={{ color: '#00ff44' }}>CONNECTED</span></p>
              <p>// INGESTION_WORKER: <span style={{ color: '#00ff44' }}>ACTIVE</span></p>
              <div style={{ marginTop: '20px', padding: '10px', border: '1px dashed #00ff44' }}>
                <p style={{ margin: 0 }}>[ SYSTEM LOGS ]</p>
                <div style={{ fontSize: '0.8rem', color: '#888', marginTop: '10px' }}>
                  Ingestion sweep completed in 1.4s...<br/>
                  Cache hit rate: 98.2%...<br/>
                  New knowledge indexed: 14 nodes...
                </div>
              </div>
            </div>
          )}

          {activeTab === 'credentials' && (
            <div style={{ opacity: 0.5 }}>
              <h3>External API Access</h3>
              <p>// Access restricted to ROOT level users.</p>
              <div style={{ display: 'flex', flexDirection: 'column', gap: '15px' }}>
                <div>
                  <label>GOOGLE_CLIENT_ID</label><br/>
                  <input type="password" value="************************" readOnly style={{ width: '100%', background: '#000', color: '#444', border: '1px solid #222', padding: '10px' }} />
                </div>
                <div>
                  <label>OLLAMA_ENDPOINT</label><br/>
                  <input type="text" value="http://ollama:11434" readOnly style={{ width: '100%', background: '#000', color: '#444', border: '1px solid #222', padding: '10px' }} />
                </div>
              </div>
            </div>
          )}
        </div>
      </div>
    </div>
  );
};

function App() {
  const [nodes, setNodes] = useState<KnowledgeNode[]>([]);
  const [isSettingsOpen, setIsSettingsOpen] = useState(false);

  useEffect(() => {
    const fetchNodes = async () => {
      try {
        const response = await axios.get(`${API_BASE}/nodes`);
        setNodes(response.data);
      } catch (error) {
        console.error('Failed to fetch nodes', error);
      }
    };
    fetchNodes();
    const interval = setInterval(fetchNodes, 5000);
    return () => clearInterval(interval);
  }, []);

  return (
    <div style={{ width: '100vw', height: '100vh', background: '#000' }}>
      <div style={{
        position: 'absolute',
        top: 30,
        right: 30,
        zIndex: 10,
      }}>
        <button
          onClick={() => setIsSettingsOpen(true)}
          style={{
            background: 'rgba(0,255,68,0.1)',
            border: '1px solid #00ff44',
            color: '#00ff44',
            padding: '10px 20px',
            fontFamily: 'monospace',
            cursor: 'pointer',
            boxShadow: '0 0 15px rgba(0,255,68,0.3)',
            pointerEvents: 'auto',
            letterSpacing: '1px'
          }}
        >
          [ SYSTEM_CONFIG ]
        </button>
      </div>

      <SettingsModal isOpen={isSettingsOpen} onClose={() => setIsSettingsOpen(false)} />

      <div style={{
        position: 'absolute',
        top: 30,
        left: 30,
        zIndex: 10,
        color: '#00ff44',
        fontFamily: "'Courier New', Courier, monospace",
        pointerEvents: 'none',
        textShadow: '0 0 10px #00ff44'
      }}>
        <h1 style={{ margin: 0, fontSize: '2.2rem', letterSpacing: '3px' }}>BRAIN_CORE.V1</h1>
        <p style={{ margin: '5px 0' }}>// NEURAL_NODES: {nodes.length}</p>
        <p style={{ margin: '5px 0' }}>// NETWORK_STATUS: ONLINE</p>
        <p style={{ margin: '5px 0' }}>// VPS_TARGET: brain.kluth.cloud</p>
      </div>
      
      <Canvas camera={{ position: [15, 15, 15], fov: 45 }}>
        <color attach="background" args={['#000']} />
        
        <ambientLight intensity={0.1} />
        <pointLight position={[10, 10, 10]} intensity={2} color="#00ff44" />
        
        <Suspense fallback={null}>
          <Stars radius={100} depth={50} count={5000} factor={4} saturation={0} fade speed={1} />
          <DataStreams />
          
          {nodes.map((node, i) => (
            <Node
              key={i}
              node={node}
              position={[
                (Math.random() - 0.5) * 20,
                (Math.random() - 0.5) * 20,
                (Math.random() - 0.5) * 20
              ]}
            />
          ))}

          <EffectComposer>
            <Bloom luminanceThreshold={0.1} luminanceSmoothing={0.9} height={300} intensity={1.5} />
            <Scanline opacity={0.1} />
            <Noise opacity={0.05} />
          </EffectComposer>
        </Suspense>

        <OrbitControls enableDamping autoRotate autoRotateSpeed={0.5} />
      </Canvas>
    </div>
  );
}

export default App;
