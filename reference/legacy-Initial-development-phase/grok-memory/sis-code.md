# directory structure

((venv) ) amoljassal@Amols-Mac-mini sis-hub % ls -F
__init__.py			Procfile
app.py				requirements.txt
config/				sis-code.md
memory/				test.py
MODULAR_OS_EXTRACTIONS.md	tests/
modules/			venv/

((venv) ) amoljassal@Amols-Mac-mini sis-hub % ls -F config 
encryption_key.key	sis_baseline_hash.txt	sis.json

((venv) ) amoljassal@Amols-Mac-mini sis-hub % ls -F  modules/ memory/  tests/                                 

memory/:
archive/	mock_ipc.json	scroll_full.db

modules/:
__init__.py		kernel_interface.py	state_awareness.py
__pycache__/		memory_logger.py	synchronizer.py
archive.py		mirror_simulator.py	verifier.py
encryptor.py		philo_explainer.py	web_search.py
entropy_detector.py	planner.py
executor.py		registry.py

tests/:
__init__.py			test_planner.py
__pycache__/			test_state_awareness.py
test_entropy_detector.py	test_verifier.py
test_executor.py



# app.py

import streamlit as st
import json
from ecdsa import SigningKey, VerifyingKey, NIST384p
import networkx as nx
from cryptography.fernet import Fernet
import matplotlib.pyplot as plt
import os
import io
import sqlite3
from PIL import Image
import threading
import shutil

# Import all modules
from modules.synchronizer import synchronizer
from modules.planner import generate_plan
from modules.verifier import verify_plan
from modules.executor import executor
from modules.memory_logger import memory_logger, verify_hash
from modules.mirror_simulator import get_simulation_preview
from modules.encryptor import encrypt_file, generate_key, decrypt_file, load_key
from modules.archive import create_snapshot, list_snapshots, restore_snapshot
from modules.entropy_detector import detect_drift
from modules.state_awareness import heal_system, check_drift_alert, vrg_gate, anchor_config, habit_anchor
from modules.philo_explainer import explain_insight
from modules.registry import register_concept, audit_registry

sis_config = {}
if os.path.exists('config/sis.json'):
    with open('config/sis.json', 'r') as f:
        sis_config = json.load(f)

st.title("Sovereign Interface System (SIS) 2.0 - Your Fortress")

st.info(anchor_config())

if 'sovereign_key' not in st.session_state:
    st.session_state.sovereign_key = SigningKey.generate(curve=NIST384p)
    st.session_state.verifying_key = st.session_state.sovereign_key.verifying_key
    st.success("Sovereign Key Generated - Your Fortress is Initialized!")
if not os.path.exists('config/encryption_key.key'):
    os.makedirs('config', exist_ok=True)
    st.session_state.encryption_key = generate_key()
    st.success("Encryption Key Generated - Secure Fortress Activated!")

def run_habit_anchor():
    alert = habit_anchor()
    if "Alert" in alert:
        st.warning(alert)
threading.Thread(target=run_habit_anchor, daemon=True).start()

philosophy_lens = st.selectbox("Choose Philosophy Lens", ["Stoic", "Vedantic", "Egoist", "Non-Dual"])
humor_enabled = st.checkbox("Enable Humor in Insights")
mood = st.selectbox("Choose Mood for Insights", ["standard", "calm", "energetic", "reflective"])
directive = st.text_input("Enter Directive (e.g., 'Plot y = sin(x)')")

def execute_directive():
    is_incoherent, gate_msg = vrg_gate(directive)
    if is_incoherent:
        st.error(gate_msg)
        return  # Now inside function
    else:
        signature = st.session_state.sovereign_key.sign(directive.encode())
        if not st.session_state.verifying_key.verify(signature, directive.encode()):
            st.error("Invalid Signature - Access Denied.")
            return  # Inside function
        plan_json = None
        try:
            decomposed = synchronizer(directive, philosophy_lens)
            plan_json, graph = generate_plan(decomposed)
            lenses = sis_config.get("lenses", {})
            lens_weight = lenses.get(philosophy_lens, {"weight": 1.0})["weight"]
            lens_node = register_concept(philosophy_lens, {"type": "Lens"}, lens_weight)
            is_drift_alert, drift_alert_msg = check_drift_alert(plan_json)
            if is_drift_alert:
                st.warning(drift_alert_msg)
                return
            else:
                is_drift, drift_msg = detect_drift(plan_json)
                if is_drift:
                    st.error(f"Drift detected: {drift_msg}")
                    return
                sim_success, sim_result = get_simulation_preview(plan_json)
                if sim_success and sim_result:
                    st.image(f"data:image/png;base64,{sim_result}", caption="Plan Simulation Preview", use_container_width=True)
                elif not sim_success:
                    st.warning(f"Simulation Warning: {sim_result}")
                valid, msg = verify_plan(plan_json, philosophy_lens)
                if valid:
                    result, img_bytes = executor(plan_json, philosophy_lens, directive, humor=humor_enabled)
                    image_path = memory_logger(directive, plan_json, result, img_bytes)
                    if image_path:
                        encrypt_status, encrypt_msg = encrypt_file(image_path)
                        if encrypt_status:
                            st.success(encrypt_msg)
                            enc_path = f"{image_path}.enc"
                            snap_status, snap_msg = create_snapshot(enc_path)
                            if snap_status:
                                st.success(snap_msg)
                            else:
                                st.warning(snap_msg)
                        else:
                            st.warning(encrypt_msg)
                    st.success(f"Plan Verified: {msg}")
                    plan_data = json.loads(plan_json)
                    st.write("Plan Steps:", plan_data["justification"])
                    st.write("Result:", result)
                    if img_bytes:
                        st.image(img_bytes, caption=f"Plot from {directive}", use_container_width=True)
                else:
                    st.error(msg)
        except Exception as e:
            st.error(f"Fortress Error: {str(e)}. Refine directive and retry.")

if st.button("Execute Directive"):
    execute_directive()  # Call function

if st.button("View Logs"):
    conn = sqlite3.connect('memory/scroll_full.db')
    c = conn.cursor()
    logs = c.execute("SELECT id, directive, result, image_path FROM logs").fetchall()
    for log in logs:
        log_id, directive, result, image_path = log
        valid, hash_msg = verify_hash(log_id)
        st.write(f"Directive: {directive}, Result: {result} {hash_msg}")
        if image_path:
            temp_path = f"{image_path}.temp"
            shutil.copy2(image_path, temp_path)
            try:
                decrypt_status, _ = decrypt_file(temp_path)
                if not decrypt_status:
                    raise Exception("Decrypt failed - assume not encrypted")
            except:
                pass
            img_stream = io.BytesIO(open(temp_path, 'rb').read())
            st.image(img_stream, caption=f"Plot from {directive}", use_container_width=True)
            os.remove(temp_path)
    conn.close()

if st.button("Audit Registry"):
    st.write("Registry Audit:", audit_registry())

st.header("Archive Management")
if st.button("List Snapshots"):
    success, snaps = list_snapshots()
    if success:
        st.write("Snapshots:", snaps)
    else:
        st.warning(snaps)
snapshot_to_restore = st.text_input("Enter Snapshot Path to Restore (e.g., memory/archive/snapshots/snapshot_20250802_120506.zip.enc)")
if st.button("Restore Snapshot"):
    success, msg = restore_snapshot(snapshot_to_restore)
    if success:
        st.success(msg)
    else:
        st.warning(msg)

st.header("Edit Axioms (System Beliefs)")
axioms = sis_config.get("axioms", [])
for i, ax in enumerate(axioms):
    st.write(f"{i+1}. {ax['name']}: {ax['description']} (Weight: {ax['weight']})")
    new_weight = st.number_input(f"Rank {ax['name']} Weight", value=ax['weight'], min_value=0.0, step=0.1, key=f"weight_{i}")
    ax['weight'] = new_weight
with open('config/sis.json', 'w') as f:
    json.dump(sis_config, f)
st.success("Axioms updated—rerun directives to see weighted effects in verification.")




# Procfile

web: streamlit run --server.port $PORT --server.headless true app.py


# test.py

import google.generativeai as genai
genai.configure(api_key="AIzaSyBacI7JsVqWpuKJhiF-oHaY-bxR5ZfZMz4")
model = genai.GenerativeModel('gemini-2.5-pro')
response = model.generate_content("Hello")
print(response.text)


# requirements.txt

altair==5.5.0
annotated-types==0.7.0
attrs==25.3.0
blinker==1.9.0
cachetools==5.5.2
certifi==2025.7.14
cffi==1.17.1
charset-normalizer==3.4.2
click==8.2.1
contourpy==1.3.3
cryptography==45.0.5
cycler==0.12.1
ecdsa==0.19.1
fonttools==4.59.0
gitdb==4.0.12
GitPython==3.1.45
google-ai-generativelanguage==0.6.15
google-api-core==2.25.1
google-api-python-client==2.177.0
google-auth==2.40.3
google-auth-httplib2==0.2.0
google-generativeai==0.8.5
googleapis-common-protos==1.70.0
grpcio==1.74.0
grpcio-status==1.71.2
httplib2==0.22.0
idna==3.10
Jinja2==3.1.6
jsonschema==4.25.0
jsonschema-specifications==2025.4.1
kaleido==0.2.1
kiwisolver==1.4.8
MarkupSafe==3.0.2
matplotlib==3.10.3
mpmath==1.3.0
narwhals==1.48.1
networkx==3.5
numpy==2.3.2
packaging==25.0
pandas==2.3.1
pillow==11.3.0
plotly==5.22.0
proto-plus==1.26.1
protobuf==5.29.5
pyarrow==21.0.0
pyasn1==0.6.1
pyasn1_modules==0.4.2
pycparser==2.22
pydantic==2.11.7
pydantic_core==2.33.2
pydeck==0.9.1
pyparsing==3.2.3
pytest==8.2.2
python-dateutil==2.9.0.post0
pytz==2025.2
referencing==0.36.2
requests==2.32.4
rpds-py==0.26.0
rsa==4.9.1
six==1.17.0
smmap==5.0.2
streamlit==1.47.1
sympy==1.14.0
tenacity==9.1.2
toml==0.10.2
tornado==6.5.1
tqdm==4.67.1
typing-inspection==0.4.1
typing_extensions==4.14.1
tzdata==2025.2
uritemplate==4.2.0
urllib3==2.5.0
watchdog==6.0.0


# sis.json

{
  "axioms": [
    "Ownness: Empower Sovereign control.",
    "Non-Duality: Static absolute, dynamic relational."
  ],
  "lenses": {
    "Stoic": {"weight": 1.5},
    "Vedantic": {"weight": 1.2},
    "Egoist": {"weight": 1.0},
    "Non-Dual": {"weight": 1.8}
  },
  "incoherent_words": ["paradox", "contradict", "infinite"]
}


# mock_ipc.json

{"source": "SIS", "target_server": "VFS_SERVER", "action": "READ_FILE", "request_id": "70925", "payload": {"path": "/mock/path"}}
{"source": "SIS", "target_server": "VFS_SERVER", "action": "READ_FILE", "request_id": "70925", "payload": {"path": "/mock/path"}}






# archive.py

import os
import shutil
from datetime import datetime
from modules.encryptor import encrypt_file, decrypt_file, load_key

def create_snapshot(encrypted_file_path=None, dest_dir='memory/archive/snapshots'):
    """Create an encrypted snapshot of a specific encrypted file."""
    if encrypted_file_path and not os.path.exists(encrypted_file_path):
        return False, f"Source file {encrypted_file_path} not found."
    if not os.path.exists('memory/archive'):
        return False, "Archive directory not found."
    if not os.path.exists(dest_dir):
        os.makedirs(dest_dir)
    
    timestamp = datetime.now().strftime('%Y%m%d_%H%M%S')
    snapshot_name = f"snapshot_{timestamp}.zip"
    snapshot_path = os.path.join(dest_dir, snapshot_name)
    
    # Create a temp directory for zipping
    temp_dir = os.path.join(dest_dir, f"temp_{timestamp}")
    os.makedirs(temp_dir, exist_ok=True)
    
    # Copy the encrypted file to temp dir with a fixed name
    if encrypted_file_path:
        temp_file_path = os.path.join(temp_dir, os.path.basename(encrypted_file_path))
        shutil.copy2(encrypted_file_path, temp_file_path)
    
    # Create zip from temp dir
    try:
        shutil.make_archive(snapshot_path.replace('.zip', ''), 'zip', temp_dir)
    except Exception as e:
        shutil.rmtree(temp_dir, ignore_errors=True)
        return False, f"Snapshot creation failed: {str(e)}"
    
    # Encrypt the snapshot
    key = load_key()
    success, message = encrypt_file(snapshot_path, key)
    
    # Clean up temp dir
    shutil.rmtree(temp_dir, ignore_errors=True)
    
    if success:
        return True, f"Snapshot {snapshot_name} created and encrypted successfully."
    return False, message

def list_snapshots(dest_dir='memory/archive/snapshots'):
    """List all encrypted snapshots."""
    if not os.path.exists(dest_dir):
        return False, "Snapshot directory not found."
    snapshots = [f for f in os.listdir(dest_dir) if f.endswith('.zip')]
    return True, snapshots if snapshots else "No snapshots found."

def restore_snapshot(snapshot_path, dest_dir='memory/archive'):
    """Restore an encrypted snapshot (decrypt and extract)."""
    if not os.path.exists(snapshot_path):
        return False, "Snapshot not found."
    key = load_key()
    # Decrypt the snapshot
    success, message = decrypt_file(snapshot_path, key)  # Fixed typo: decrypt_file
    if success:
        # Extract the decrypted zip
        try:
            shutil.unpack_archive(snapshot_path.replace('.zip', ''), dest_dir, 'zip')
            os.remove(snapshot_path.replace('.zip', ''))  # Clean up decrypted file
            return True, f"Snapshot {snapshot_path} restored successfully."
        except Exception as e:
            return False, f"Snapshot restoration failed: {str(e)}"
    return False, message





# encryptor.py

from cryptography.fernet import Fernet
import os
import shutil  # For copy

def generate_key():
    """Generate a new encryption key and save it to a file."""
    os.makedirs('config', exist_ok=True)  # Create config/ if missing
    key = Fernet.generate_key()
    with open('config/encryption_key.key', 'wb') as key_file:
        key_file.write(key)
    return key

def load_key():
    """Load the encryption key from the file."""
    return open('config/encryption_key.key', 'rb').read()

def encrypt_file(file_path, key=None):
    """Encrypt a copy of the file using AES, keep original, return original path."""
    if not os.path.exists(file_path):
        return False, "File not found."
    key = key or load_key()
    cipher_suite = Fernet(key)
    enc_path = f"{file_path}.enc"
    shutil.copy2(file_path, enc_path)  # Always copy original to .enc
    with open(enc_path, 'rb') as file:
        file_data = file.read()
    encrypted_data = cipher_suite.encrypt(file_data)
    with open(enc_path, 'wb') as file:
        file.write(encrypted_data)
    return True, f"File {enc_path} encrypted successfully (original {file_path} intact)."

def decrypt_file(file_path, key=None):
    """Decrypt a file using AES."""
    if not os.path.exists(file_path):
        return False, "File not found."
    key = key or load_key()
    cipher_suite = Fernet(key)
    with open(file_path, 'rb') as file:
        encrypted_data = file.read()
    decrypted_data = cipher_suite.decrypt(encrypted_data)
    with open(file_path, 'wb') as file:
        file.write(decrypted_data)
    return True, f"File {file_path} decrypted successfully."




# entropy_detector.py 

import json
import networkx as nx

def detect_drift(plan_json, prev_plan_json=None):
    plan = json.loads(plan_json)
    G = nx.DiGraph(plan.get("graph", []))
    # Fixed: Use (nodes + edges) / 30.0 for proper normalization; cap at 1.0 like verifier
    drift_score = (len(G.nodes) + len(G.edges)) / 30.0
    drift_score = min(drift_score, 1.0)  # Cap to avoid extremes
    if prev_plan_json:
        prev_G = nx.DiGraph(json.loads(prev_plan_json).get("graph", []))
        drift_score += nx.graph_edit_distance(G, prev_G)  # Compare to prior (optional)
    return drift_score > 0.5, f"Drift score: {drift_score:.2f}" if drift_score > 0.5 else "Coherent"




# executor.py

import sympy as sp
import re
from modules.philo_explainer import explain_insight
import matplotlib.pyplot as plt
import numpy as np
import io
import json
from modules.web_search import web_search
import streamlit as st  # Added for st.progress

def executor(plan_json, lens, task, humor=False):
    """Execute the verified plan, handling math solving/plotting and API integration (Kernel Interface + Lumencore)."""
    result = "Execution complete: Result simulated."
    task_lower = task.lower()
    if "solve" in task_lower or ("=" in task and "plot" not in task_lower and "y" not in task_lower):
        try:
            eq_str = re.sub(r'(?i)solve\s*', '', task).strip()
            eq_str = re.sub(r'\^', '**', eq_str)
            if '=' in eq_str:
                left, right = eq_str.split('=', 1)
                eq_str = f"({left.strip()}) - ({right.strip()})"
            eq = sp.sympify(eq_str)
            solutions = sp.solve(eq)
            result = f"Solutions: {solutions}. {lens} Insight - Equations as balanced virtues in the fortress of reason."
            # Feedback Loop: Refine result (limited 3 iters, e.g., simplify solutions)
            max_iters = 3
            for i in range(max_iters):
                st.progress((i + 1) / max_iters, text="Refining solutions...")  # Project Dock: Timeline bar
                if isinstance(solutions, list):
                    solutions = sp.simplify(sp.Matrix(solutions))
                    result = f"Refined Solutions: {solutions}."
                else:
                    break
            result = explain_insight(result, lens, dual=True, humor=humor)
            return result, None
        except Exception as e:
            result = f"Math solving error: {str(e)} - Refine directive (e.g., 'Solve x**2 - 5*x + 6 = 0')."
            return result, None
    elif "plot" in task_lower or "y =" in task_lower:
        try:
            fn_str = re.sub(r'(?i)plot\s*y\s*=\s*', '', task).strip()
            fn_str = re.sub(r'\^', '**', fn_str)
            fn = sp.lambdify(sp.symbols('x'), sp.sympify(fn_str), 'numpy')
            x_vals = np.linspace(-10, 10, 100)
            y_vals = fn(x_vals)
            fig, ax = plt.subplots()
            ax.plot(x_vals, y_vals)
            ax.set_title(f"Plot of {fn_str}")
            ax.set_xlabel("x")
            ax.set_ylabel("y")
            img_buffer = io.BytesIO()
            fig.savefig(img_buffer, format='png')
            img_buffer.seek(0)
            img_bytes = img_buffer.getvalue()
            plt.close(fig)
            result = f"Plotted {fn_str}. {lens} Insight - Functions as harmonious paths in the fortress of reason."
            result = explain_insight(result, lens, dual=True, humor=humor)
            return result, img_bytes
        except Exception as e:
            result = f"Math plotting error: {str(e)} - Refine directive (e.g., 'Plot y = sin(x)')."
            return result, None
    else:
        if "find" in task_lower or "search" in task_lower:
            plan_json, G = web_search(task, lens)
            result = json.loads(plan_json)["output"]
            result = explain_insight(result, lens, dual=True, humor=humor)
            return result, None
        result = f"Simulated non-math directive: {task}. {lens} Insight - Actions as disciplined paths in the fortress."
        result = explain_insight(result, lens, dual=True, humor=humor)
        return result, None




# kernel_interface.py

import json
import os  # For mock file/pipe simulation

def send_ipc_message(target_server, action, payload):
    """Send structured JSON IPC message to mocked kernel server (e.g., write to file/pipe)."""
    message = {
        "source": "SIS",
        "target_server": target_server,
        "action": action,
        "request_id": str(os.getpid()),  # Unique ID for tracing
        "payload": payload
    }
    # Mock: Write to a file simulating pipe/device (e.g., /dev/sis_ipc)
    mock_pipe = 'memory/mock_ipc.json'  # Append-only for testing
    with open(mock_pipe, 'a') as f:
        f.write(json.dumps(message) + '\n')
    return True, "IPC sent successfully (mocked)."

def receive_ipc_response(request_id):
    """Mock receive response from kernel server."""
    # Simulate reading response (e.g., success echo)
    return json.dumps({"status": "success", "request_id": request_id, "result": "Mock response"})




# memory_logger.py

import sqlite3
import os
import time
import hashlib  # For hash protection

def add_column_if_missing(conn, table, column, type):
    """Dynamically add column if not exists (migration helper)."""
    c = conn.cursor()
    try:
        c.execute(f"ALTER TABLE {table} ADD COLUMN {column} {type}")
        conn.commit()
    except sqlite3.OperationalError:  # Column already exists
        pass

def memory_logger(directive, plan, result, img_bytes=None):
    """Log directives, plans, and results to scroll_full.db with image paths (Chronicler + Lumencore)."""
    conn = sqlite3.connect('memory/scroll_full.db')
    c = conn.cursor()
    c.execute('''CREATE TABLE IF NOT EXISTS logs 
                 (id INTEGER PRIMARY KEY, directive TEXT, plan TEXT, result TEXT, image_path TEXT)''')  # Base schema
    # Dynamic migration for new columns
    add_column_if_missing(conn, 'logs', 'timestamp', 'TEXT')
    add_column_if_missing(conn, 'logs', 'log_hash', 'TEXT')
    image_path = None
    timestamp = time.strftime("%Y-%m-%d %H:%M:%S")
    if img_bytes:
        archive_dir = 'memory/archive'
        os.makedirs(archive_dir, exist_ok=True)
        img_timestamp = time.time()
        image_path = f"{archive_dir}/log_{int(img_timestamp)}.png"
        with open(image_path, 'wb') as f:
            f.write(img_bytes)
    # Compute hash for protection (directive + str(plan) + result + timestamp)
    plan_str = str(plan)
    log_data = directive + plan_str + result + timestamp
    log_hash = hashlib.sha256(log_data.encode()).hexdigest()
    c.execute("INSERT INTO logs (directive, plan, result, image_path, timestamp, log_hash) VALUES (?, ?, ?, ?, ?, ?)", 
              (directive, plan_str, result, image_path, timestamp, log_hash))  # Store str(plan)
    conn.commit()
    conn.close()
    return image_path

def verify_hash(log_id):
    """Verify log integrity with hash on retrieval."""
    conn = sqlite3.connect('memory/scroll_full.db')
    c = conn.cursor()
    log = c.execute("SELECT directive, plan, result, timestamp, log_hash FROM logs WHERE id = ?", (log_id,)).fetchone()
    if log:
        directive, plan_str, result, timestamp, stored_hash = log
        if stored_hash is None:
            return True, "Legacy log: No hash—pre-protection entry."
        log_data = directive + plan_str + result + timestamp
        computed_hash = hashlib.sha256(log_data.encode()).hexdigest()
        if computed_hash == stored_hash:
            return True, "Log verified: Hash matches."
        return False, "Log tampered: Hash mismatch."
    return False, "Log not found."




# mirror_simulator.py

import json
import networkx as nx
import matplotlib.pyplot as plt
import io
import base64

def simulate_plan(plan_json):
    try:
        plan = json.loads(plan_json)
        G = nx.DiGraph(plan.get("graph", []))
        if not G.nodes:
            return False, "Simulation failed: Empty plan graph."
        if any(G.has_edge(n, n) for n in G.nodes()):
            return False, "Simulation failed: Self-loop detected."
        plt.figure(figsize=(10, 6))
        pos = nx.spring_layout(G)
        nx.draw(G, pos, with_labels=True, node_color='lightgreen', edge_color='purple', 
                node_size=2000, font_size=10, font_weight='bold', arrows=True)
        plt.title("Plan Simulation - Dry Run Preview")
        img_buffer = io.BytesIO()
        plt.savefig(img_buffer, format='png')
        img_buffer.seek(0)
        base64_data = base64.b64encode(img_buffer.getvalue()).decode('utf-8')
        plt.close()
        return True, base64_data
    except Exception as e:
        return False, f"Simulation error: {str(e)}"

def get_simulation_preview(plan_json):
    return simulate_plan(plan_json)




# philo_explainer.py

def explain_insight(result, lens, dual=False, directive="", humor=False):
    insight = f"{lens} Insight - {result}"
    if directive:
        insight += f" applied to '{directive}'"
    if dual:
        static = f" Static: Absolute coherence in {lens}."
        dynamic = f" Dynamic: Relational application in world."
        insight += f"{static}{dynamic}"
    if humor:
        insight += " (With a twist: Like a cosmic joke where the punchline is enlightenment!)"
    return insight




# planner.py

import re
import networkx as nx
import json
from ollama import Client  # Import Ollama

def generate_plan(task):
    client = Client()  # Local Ollama client
    prompt = f"Parse this directive into 3-5 steps, each step must be exactly formatted as 'Step: description. {task['lens']} Insight: explanation': {task['task']}. View through {task['lens']} lens. Output as numbered list only (e.g., 1. Step: desc. Insight: expl.)."

    try:
        response = client.generate(model='llama3:8b', prompt=prompt)
        content = response['response']
    except:
        content = ""  # Fallback if Ollama fails

    # Parse steps
    steps = []
    for line in content.split('\n'):
        if line.strip() and "Step:" in line and "Insight:" in line:
            step = re.sub(r'^\d+\.\s*|[*#\-\.]', '', line).strip()
            steps.append(step)

    if not steps:
        steps = ["Action1", "Process"]

    # Build graph
    G = nx.DiGraph()
    prev = "Start"
    for step in steps:
        G.add_edge(prev, step)
        prev = step
    G.add_edge(prev, "End")

    # Justification with Ownness
    justification = f"AI-parsed vector from intent to execution, viewed through {task['lens']} lens, to empower the Sovereign's ownness and control. Steps: {', '.join(steps)}."
    plan = {"graph": list(G.edges()), "justification": justification}
    return json.dumps(plan), G




# registry.py

class ConceptNode:
    def __init__(self, name, properties=None, weight=1.0):
        self.name = name
        self.properties = properties or {}
        self.weight = weight
        self.edges = []
        self.domain = self.properties.get("domain", "None")  # Add domain for Gems

class RelationshipEdge:
    def __init__(self, start_node, end_node, properties=None, weight=1.0):
        self.start_node = start_node
        self.end_node = end_node
        self.properties = properties or {}
        self.weight = weight

def stabilize_canvas(properties):
    if "type" not in properties or properties["type"] not in ["Lens", "Gem", "Node"]:
        raise ValueError("Invalid canvas type—must be Lens, Gem, or Node.")
    if not isinstance(properties, dict):
        raise ValueError("Properties must be a dict.")
    return True, "Canvas stabilized."

def register_concept(name, properties, weight=1.0):
    stable, msg = stabilize_canvas(properties)
    if not stable:
        raise ValueError(msg)
    return ConceptNode(name, properties, weight)

def audit_registry(nodes=None, edges=None):
    nodes = nodes or []
    edges = edges or []
    total_weight = sum(n.weight for n in nodes) + sum(e.weight for e in edges)
    domains = [n.domain for n in nodes if n.properties.get("type") == "Gem"]
    return f"Registry Audit: {len(nodes)} nodes, {len(edges)} edges. Total Anchor Weight: {total_weight:.2f}. Domains: {', '.join(domains)}. All stabilized."





# state_awareness.py

import os
import subprocess
from modules.entropy_detector import detect_drift
import time
import hashlib  # Added for hashing

class CognitiveState:
    """Defines states of awareness, inspired by Geometric Imperative as progressive coordinates."""
    LEARNING = 0
    UNDERSTANDING = 1
    REALIZATION = 2

class CognitiveEntity:
    def __init__(self):
        self.state = CognitiveState.LEARNING

    def reflect(self, coherent):
        """Advance state based on plan coherence, ensuring stability (Resilient Fortress)."""
        if coherent and self.state < CognitiveState.REALIZATION:
            self.state += 1
        return self.state

def vrg_gate(directive):
    """Filter incoherent inputs like kernel interrupts (Resilient Fortress)."""
    incoherent_words = ["paradox", "contradict", "infinite"]
    return any(word in directive.lower() for word in incoherent_words), "Gated: Incoherent input detected" if any else "Coherent"

def heal_system(fault_msg, plan_json=None):
    """Handle errors like kernel ISRs, with rollback or retry (Resilient Fortress)."""
    try:
        if plan_json:
            is_drift, _ = detect_drift(plan_json)
            if is_drift:
                subprocess.run(["git", "checkout", "phase-4-stable"], stderr=subprocess.DEVNULL)
                return "Healed: Reverted to phase-4-stable due to drift."
        subprocess.run(["git", "checkout", "phase-4-stable"], stderr=subprocess.DEVNULL)
        return "Healed: Reverted to baseline tag."
    except Exception as e:
        return f"Heal failed: {fault_msg} - {str(e)}"

def check_drift_alert(plan_json):
    """Raise alert for high drift, mimicking kernel interrupt handling (Geometric Imperative)."""
    is_drift, drift_msg = detect_drift(plan_json)
    if is_drift:
        return True, f"Drift Alert: {drift_msg}"
    return False, "System Stable"

def heartbeat_protocol(timeout_seconds=300):
    """Simulate timeout to standby state if no valid command (Resilient Fortress)."""
    start_time = time.time()
    while time.time() - start_time < timeout_seconds:
        time.sleep(1)
    return "Heartbeat Timeout: Entering standby—refusing new actions until re-authenticated."

def anchor_config(config_path='config/sis.json', baseline_tag='phase-4-stable'):
    """Anchor config file to prevent unintended changes, using hash checks and Git revert on mismatch."""
    if os.environ.get('DEV_MODE') == '1':
        return "Dev mode: Anchoring skipped—edits persist."
    if not os.path.exists(config_path):
        return "Config not found—no anchor applied."
    with open(config_path, 'rb') as f:
        current_hash = hashlib.sha256(f.read()).hexdigest()
    try:
        subprocess.run(["git", "checkout", baseline_tag, "--", config_path], stderr=subprocess.DEVNULL)
        return "Config anchored: Reverted to baseline if changed."
    except:
        return "Anchoring failed—manual check recommended."

def habit_anchor(plan_json=None, interval_seconds=60):
    """Anchor habits with recurring entropy checks and basic recovery alerts."""
    last_check = time.time()
    while True:
        if time.time() - last_check >= interval_seconds:
            if plan_json:
                is_drift, msg = detect_drift(plan_json)
                if is_drift:
                    return f"Habit Alert: Drift detected ({msg})—recommend recovery timeline: Review last 3 logs."
            last_check = time.time()
        time.sleep(1)
    return "Habits anchored: No drift in timeline."






# synchronizer.py

def synchronizer(directive, philosophy_lens='Stoic'):
    """Decompose and route directives with a philosophical lens (Genesis + Lumencore)."""
    return {'task': directive, 'lens': philosophy_lens}



# verifier.py

import json
import networkx as nx
import hashlib
import os

class CorePrinciples:
    def __call__(self, func):
        def wrapper(*args, **kwargs):
            if True:
                return func(*args, **kwargs)
            return False, "Misaligned with principles"
        return wrapper

def load_axioms(config_path='config/sis.json'):
    if not os.path.exists(config_path):
        return []
    with open(config_path, 'r') as f:
        data = json.load(f)
    return data.get("axioms", [])

def quarantine_plan(plan_json):
    axioms = load_axioms()
    if not axioms:
        return True, "No axioms—plan elevated without quarantine."
    plan = json.loads(plan_json)
    justification = plan.get("justification", "").lower()
    reso_hash = hashlib.sha256(justification.encode()).hexdigest()[:8]
    keyword_match = False
    matched_weights = 0.0
    for ax in axioms:
        if isinstance(ax, dict):
            desc = ax["description"].lower()
            weight = ax.get("weight", 1.0)
        else:  # Handle str axioms
            desc = str(ax).lower()
            weight = 1.0  # Default weight
        if any(word in justification for word in desc.split()):
            keyword_match = True
            matched_weights += weight
    if keyword_match:
        return True, f"Plan elevated: Resonance hash {reso_hash} matches axioms (weighted {matched_weights:.2f})."
    return False, f"Quarantined: No resonance (hash {reso_hash}, weighted {matched_weights:.2f}) with axioms—refine."

@CorePrinciples()
def verify_plan(plan_json, lens):
    try:
        elevated, msg = quarantine_plan(plan_json)
        if not elevated:
            return False, msg
        plan = json.loads(plan_json)
        G = nx.DiGraph(plan.get("graph", []))
        num_nodes = len(G.nodes)
        num_edges = len(G.edges)
        entropy_score = min((num_nodes + num_edges) / 30.0, 1.0)
        empowers_ownness = any(word in plan.get("justification", "").lower() for word in ["empower", "ownness", "sovereign", "control"])
        if any(G.has_edge(n, n) for n in G.nodes()):
            return False, "Paradox detected: Self-loop in plan"
        has_cycle = False
        try:
            next(nx.simple_cycles(G))
            has_cycle = True
        except StopIteration:
            pass
        if has_cycle:
            return False, f"Codex Violation: Cycle detected - lacks geometric imperative ({lens} critique)."
        if not empowers_ownness:
            return False, f"Codex Violation: No Ownness - refine for empowerment ({lens} critique)."
        if entropy_score > 0.5:
            return False, f"High entropy: {entropy_score:.2f} - too chaotic ({lens} critique)."
        density = nx.density(G)
        if density > 0.8:
            return False, "ERA Violation: Excessive resonance density - too interconnected"
        return True, f"Verified: Codex alignment (Ownness empowered). Entropy: {entropy_score:.2f}. {msg}"
    except Exception as e:
        return False, f"Verification error: {str(e)}"





# web_search.py

import json
import networkx as nx
from modules.registry import register_concept

def web_search(directive, lens):
    """
    Mock Web 3.0 search: Parse directive, return curated results with verified links.
    Future: Integrate real API (e.g., google-api-python-client).
    """
    # Tailored mock results based on directive keywords
    lower_directive = directive.lower()
    if "budget" in lower_directive:
        mock_results = [
            {"title": "Mint - Secure Budget App", "url": "https://mint.intuit.com", "snippet": "Track expenses securely with encryption and alerts."},
            {"title": "YNAB - You Need A Budget", "url": "https://www.youneedabudget.com", "snippet": "Zero-based budgeting with bank-grade security."},
            {"title": "PocketGuard - Budget Tracker", "url": "https://pocketguard.com", "snippet": "AI-powered budgeting with fraud protection."}
        ]
    elif "secure" in lower_directive:
        mock_results = [
            {"title": "LastPass - Secure Password Manager", "url": "https://lastpass.com", "snippet": "End-to-end encrypted password storage."},
            {"title": "NordVPN - Secure VPN Tool", "url": "https://nordvpn.com", "snippet": "Encrypted internet access for privacy."}
        ]
    else:
        mock_results = [
            {"title": "Sample Result 1", "url": "https://example.com/1", "snippet": "Secure resource for directive."},
            {"title": "Sample Result 2", "url": "https://example.com/2", "snippet": "Verified info aligned with intent."}
        ]
    
    # Build graph for verification
    G = nx.DiGraph()
    G.add_node("Start")
    for i, result in enumerate(mock_results, 1):
        node = f"Result_{i}"
        G.add_node(node, title=result["title"], url=result["url"], snippet=result["snippet"])
        G.add_edge("Start", node)
    G.add_edge(f"Result_{len(mock_results)}", "End")
    
    # Register search context as "Gem"
    search_node = register_concept(f"Search_{directive[:20]}", {"type": "Gem", "lens": lens})
    
    # Format output with links and philosophical insight
    output = f"Web Search Results for '{directive}':\n"
    for i, result in enumerate(mock_results, 1):
        output += f"{i}. [{result['title']}]({result['url']}) - {result['snippet']}\n"
    output += f"{lens} Insight: Results curated as paths to wisdom, empowering Sovereign control."
    
    # Add Ownness-aligned justification
    justification = f"Web search vectorized to empower the Sovereign's ownness and control via {lens} lens. Results: {', '.join([r['title'] for r in mock_results])}. Interlinked with {search_node.name}."
    
    # Serialize plan to JSON string
    plan = {"graph": list(G.edges()), "results": mock_results, "output": output, "justification": justification}
    return json.dumps(plan), G


# test_entropy_detector.py

import pytest
from modules.entropy_detector import detect_drift
import json

def test_detect_drift_low():
    # Mock simple plan (low drift)
    plan_json = '{"graph": [["Start", "End"]], "justification": "test"}'
    is_drift, msg = detect_drift(plan_json)
    assert not is_drift, f"Should not detect drift: {msg}"

def test_detect_drift_high():
    # Mock complex plan (high drift)
    plan_json = '{"graph": [["Start", "Step1"], ["Step1", "Step2"], ["Step2", "Step3"], ["Step3", "Step4"], ["Step4", "Step5"], ["Step5", "Step6"], ["Step6", "Step7"], ["Step7", "Step8"], ["Step8", "Step9"], ["Step9", "End"]], "justification": "test"}'
    is_drift, msg = detect_drift(plan_json)
    assert is_drift, f"Should detect drift: {msg}"




# test_executor.py

import pytest
from modules.executor import executor
from modules.philo_explainer import explain_insight

# Mock plan_json (minimal, as executor uses task directly)
PLAN_JSON = '{"graph": [["Start", "End"]], "justification": "test"}'

def test_executor_plot():
    result, img_bytes = executor(PLAN_JSON, "Stoic", "Plot y = sin(x)")
    assert "Plotted" in result, f"Plot failed: {result}"
    assert img_bytes is not None, "No image bytes returned"

def test_executor_solve():
    result, img_bytes = executor(PLAN_JSON, "Stoic", "Solve x^2 - 4 = 0")
    assert "Solutions" in result, f"Solve failed: {result}"
    assert img_bytes is None, "Unexpected image bytes for solve"

def test_executor_find():
    result, img_bytes = executor(PLAN_JSON, "Stoic", "Find secure budget tools")
    assert "Web Search Results" in result, f"Find failed: {result}"
    assert img_bytes is None, "Unexpected image bytes for find"




# test_planner.py

import pytest
from modules.planner import generate_plan
import json

def test_generate_plan():
    task = {"task": "Find secure budget tools", "lens": "Stoic"}
    plan_json, G = generate_plan(task)
    plan = json.loads(plan_json)
    assert "graph" in plan, "No graph in plan"
    assert len(G.nodes()) >= 2, "Graph too small"
    assert any(word in plan["justification"].lower() for word in ["empower", "ownness", "sovereign", "control"]), "Missing Ownness in justification"





# test_state_awareness.py

import pytest
from modules.state_awareness import vrg_gate, heal_system

def test_vrg_gate_coherent():
    directive = "Plot y = sin(x)"
    is_incoherent, msg = vrg_gate(directive)
    assert not is_incoherent, f"Should be coherent: {msg}"

def test_vrg_gate_incoherent():
    directive = "Solve infinite paradox"
    is_incoherent, msg = vrg_gate(directive)
    assert is_incoherent, f"Should gate incoherent: {msg}"

def test_heal_system_success(monkeypatch):
    # Mock subprocess.run to simulate success
    def mock_run(cmd):
        return None  # No error
    monkeypatch.setattr("subprocess.run", mock_run)
    result = heal_system("Test fault")
    assert "Healed: Reverted to baseline tag." in result, "Heal should succeed"

def test_heal_system_failure(monkeypatch):
    # Mock subprocess.run to raise exception
    def mock_run(cmd):
        raise Exception("Git error")
    monkeypatch.setattr("subprocess.run", mock_run)
    result = heal_system("Test fault")
    assert "Heal failed" in result, "Heal should handle failure"






# test_verifier.py

import pytest
from modules.verifier import verify_plan

def test_verify_plan_valid():
    # Mock a simple valid plan
    plan_json = '{"graph": [["Start", "Step1"], ["Step1", "End"]], "justification": "empower sovereign"}'
    valid, msg = verify_plan(plan_json, "Stoic")
    assert valid, f"Should be valid: {msg}"

def test_verify_plan_high_entropy():
    # Mock a complex plan with high entropy
    plan_json = '{"graph": [["Start", "Step1"], ["Step1", "Step2"], ["Step2", "Step3"], ["Step3", "Step4"], ["Step4", "End"]], "justification": "test"}'
    valid, msg = verify_plan(plan_json, "Stoic")
    assert not valid, f"Should reject high entropy: {msg}"

def test_verify_plan_cycle():
    # Mock plan with cycle
    plan_json = '{"graph": [["Start", "Step1"], ["Step1", "Step2"], ["Step2", "Start"]], "justification": "test"}'
    valid, msg = verify_plan(plan_json, "Stoic")
    assert not valid, f"Should reject cycle: {msg}"

def test_verify_plan_self_loop():
    # Mock plan with self-loop
    plan_json = '{"graph": [["Start", "Start"]], "justification": "test"}'
    valid, msg = verify_plan(plan_json, "Stoic")
    assert not valid, f"Should reject self-loop: {msg}"

