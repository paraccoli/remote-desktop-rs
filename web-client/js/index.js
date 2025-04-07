/**
 * リモートデスクトップWebクライアント
 * 
 * このスクリプトは、リモートデスクトップのWebクライアントの動作を制御します。
 * WebSocketまたはWebRTCを使用してサーバーに接続し、スクリーンショットを表示し、
 * ユーザー入力（マウス・キーボード）を送信します。
 */

(function() {
  // 設定のデフォルト値
  const DEFAULT_CONFIG = {
    serverHost: 'localhost',
    serverPort: 9090,
    useTLS: false,
    preferWebRTC: true,
    quality: 50,
    updateInterval: 100,
    autoReconnect: true,
    maxReconnectAttempts: 5,
    reconnectDelay: 2000,
    debugMode: false
  };
  
  // グローバル変数
  let config = { ...DEFAULT_CONFIG };
  let isConnected = false;
  let isConnecting = false;
  let connection = null;
  let reconnectCount = 0;
  let lastFrameTime = 0;
  let currentFPS = 0;
  let lastMouseMoveTime = 0;
  let lastTouchMoveTime = 0;
  let lastMousePos = { x: 0, y: 0 };
  let latestImageData = null;
  let webrtcPeerConnection = null;
  let keepAliveInterval = null;
  let updateInterval = null;
  let performanceStats = {
    latency: 0,
    avgLatency: 0,
    totalLatency: 0,
    latencyCount: 0,
    fps: 0,
    dataReceived: 0,
    dataSent: 0
  };
  
  // UI要素
  let canvasElement;
  let canvasContext;
  let statusElement;
  let connectButton;
  let disconnectButton;
  let qualitySlider;
  let fpsDisplay;
  let latencyDisplay;
  
  /**
   * 初期化処理
   */
  function initialize() {
    // ページ読み込み完了時の処理
    document.addEventListener('DOMContentLoaded', () => {
      // UI要素の参照を取得
      canvasElement = document.getElementById('remote-screen');
      canvasContext = canvasElement ? canvasElement.getContext('2d') : null;
      statusElement = document.getElementById('status');
      connectButton = document.getElementById('connect-button');
      disconnectButton = document.getElementById('disconnect-button');
      qualitySlider = document.getElementById('quality-slider');
      fpsDisplay = document.getElementById('fps-display');
      latencyDisplay = document.getElementById('latency-display');
      
      // イベントリスナーを設定
      if (connectButton) {
        connectButton.addEventListener('click', connect);
      }
      
      if (disconnectButton) {
        disconnectButton.addEventListener('click', disconnect);
      }
      
      if (qualitySlider) {
        qualitySlider.addEventListener('change', handleQualityChange);
      }
      
      // キャンバスのイベントリスナーをセットアップ
      setupCanvasListeners();
      
      // 保存された設定を読み込む
      loadConfig();
      
      // UI状態を更新
      updateUI();
      
      console.log('初期化完了');
    });
    
    // WebAssemblyモジュールの初期化
    // このセクションでは、Rustから生成されたWasmモジュールを初期化します
    // init()は別途生成されるwasm-bindgenコードによって提供されるはずです
    if (typeof window.wasm_bindgen !== 'undefined') {
      window.wasm_bindgen('./pkg/remote_desktop_rs_web_client_bg.wasm')
        .then(module => {
          window.wasm = module;
          if (module.initialize) {
            module.initialize()
              .then(() => {
                console.log('WebAssemblyモジュールを初期化しました');
              })
              .catch(err => {
                console.error('WebAssemblyモジュールの初期化に失敗しました:', err);
              });
          }
        })
        .catch(err => {
          console.error('WebAssemblyモジュールの読み込みに失敗しました:', err);
        });
    }
  }
  
  /**
   * 設定を読み込む
   */
  function loadConfig() {
    try {
      const savedConfig = localStorage.getItem('remoteDesktopConfig');
      if (savedConfig) {
        config = { ...DEFAULT_CONFIG, ...JSON.parse(savedConfig) };
        console.log('設定を読み込みました', config);
        
        // UIに設定を反映
        document.getElementById('server-host').value = config.serverHost;
        document.getElementById('server-port').value = config.serverPort;
        document.getElementById('use-tls').checked = config.useTLS;
        document.getElementById('prefer-webrtc').checked = config.preferWebRTC;
        qualitySlider.value = config.quality;
      }
    } catch (error) {
      console.error('設定の読み込みに失敗しました:', error);
    }
  }
  
  /**
   * 設定を保存する
   */
  function saveConfig() {
    try {
      localStorage.setItem('remoteDesktopConfig', JSON.stringify(config));
      console.log('設定を保存しました');
    } catch (error) {
      console.error('設定の保存に失敗しました:', error);
    }
  }
  
  /**
   * 接続処理
   */
  function connect() {
    if (isConnected || isConnecting) return;
    
    // フォームから接続情報を取得
    const host = document.getElementById('server-host').value;
    const port = parseInt(document.getElementById('server-port').value, 10);
    const useTLS = document.getElementById('use-tls').checked;
    const preferWebRTC = document.getElementById('prefer-webrtc').checked;
    const username = document.getElementById('username').value;
    const password = document.getElementById('password').value;
    
    // 設定を更新
    config.serverHost = host;
    config.serverPort = port;
    config.useTLS = useTLS;
    config.preferWebRTC = preferWebRTC;
    
    // 設定を保存
    saveConfig();
    
    // 接続フラグを設定
    isConnecting = true;
    reconnectCount = 0;
    
    // UI状態を更新
    updateStatus('接続中...');
    updateUI();
    
    try {
      // WebRTCを試みる
      if (preferWebRTC && isWebRTCSupported()) {
        console.log('WebRTCでの接続を試みます');
        // WebRTC接続処理（シグナリングサーバーを使用）
        // 実際の実装はサーバー側と連携する必要があります
        connectWithWebRTC();
      } else {
        console.log('WebSocketでの接続を試みます');
        // WebSocket接続処理
        connectWithWebSocket();
      }
    } catch (error) {
      console.error('接続エラー:', error);
      handleConnectionError('接続の初期化に失敗しました: ' + error.message);
    }
  }
  
  /**
   * WebSocketで接続
   */
  function connectWithWebSocket() {
    const protocol = config.useTLS ? 'wss' : 'ws';
    const wsUrl = `${protocol}://${config.serverHost}:${config.serverPort}/ws`;
    
    console.log(`WebSocketに接続: ${wsUrl}`);
    
    try {
      const socket = new WebSocket(wsUrl);
      
      // 接続成功時の処理
      socket.onopen = function() {
        console.log('WebSocket接続が確立されました');
        connection = {
          type: 'websocket',
          socket: socket
        };
        
        isConnected = true;
        isConnecting = false;
        
        // 認証情報があれば送信
        if (document.getElementById('username').value) {
          sendAuthInfo();
        } else {
          // 認証不要の場合は直接スクリーンショット要求
          requestScreenshot();
        }
        
        // 定期的な更新を開始
        startPeriodicUpdates();
        
        // UI状態を更新
        updateStatus('接続しました');
        updateUI();
      };
      
      // エラー発生時の処理
      socket.onerror = function(error) {
        console.error('WebSocketエラー:', error);
        handleConnectionError('接続エラーが発生しました');
      };
      
      // 接続切断時の処理
      socket.onclose = function(event) {
        console.log('WebSocket接続が切断されました:', event.code, event.reason);
        
        if (isConnected) {
          handleDisconnect();
        } else if (isConnecting) {
          handleConnectionError('接続に失敗しました');
        }
      };
      
      // メッセージ受信時の処理
      socket.onmessage = function(event) {
        handleWebSocketMessage(event);
      };
    } catch (error) {
      console.error('WebSocket接続エラー:', error);
      handleConnectionError('WebSocketの作成に失敗しました: ' + error.message);
    }
  }
  
  /**
   * WebRTCで接続
   */
  function connectWithWebRTC() {
    // RTCPeerConnectionの設定
    const servers = {
      iceServers: [
        { urls: 'stun:stun.l.google.com:19302' }
      ]
    };
    
    try {
      // RTCPeerConnectionを作成
      webrtcPeerConnection = new RTCPeerConnection(servers);
      
      // データチャネルを作成
      const dataChannel = webrtcPeerConnection.createDataChannel('remoteDesktop');
      
      // データチャネルイベントハンドラ
      dataChannel.onopen = function() {
        console.log('WebRTCデータチャネルが開かれました');
        connection = {
          type: 'webrtc',
          peerConnection: webrtcPeerConnection,
          dataChannel: dataChannel
        };
        
        isConnected = true;
        isConnecting = false;
        
        // 認証情報があれば送信
        if (document.getElementById('username').value) {
          sendAuthInfo();
        } else {
          // 認証不要の場合は直接スクリーンショット要求
          requestScreenshot();
        }
        
        // 定期的な更新を開始
        startPeriodicUpdates();
        
        // UI状態を更新
        updateStatus('WebRTC接続しました');
        updateUI();
      };
      
      dataChannel.onclose = function() {
        console.log('WebRTCデータチャネルが閉じられました');
        handleDisconnect();
      };
      
      dataChannel.onerror = function(error) {
        console.error('WebRTCデータチャネルエラー:', error);
        handleConnectionError('WebRTCデータチャネルでエラーが発生しました');
      };
      
      dataChannel.onmessage = function(event) {
        handleWebRTCMessage(event);
      };
      
      // WebRTCのシグナリング処理（実際の実装はバックエンドと連携する必要があります）
      // ここではHTTPリクエストでシグナリングを行う例を示します
      webrtcPeerConnection.createOffer()
        .then(offer => webrtcPeerConnection.setLocalDescription(offer))
        .then(() => {
          // シグナリングサーバーにオファーを送信
          const protocol = config.useTLS ? 'https' : 'http';
          const signalingUrl = `${protocol}://${config.serverHost}:${config.serverPort}/rtc-signaling`;
          
          return fetch(signalingUrl, {
            method: 'POST',
            headers: {
              'Content-Type': 'application/json'
            },
            body: JSON.stringify({
              type: 'offer',
              sdp: webrtcPeerConnection.localDescription
            })
          });
        })
        .then(response => response.json())
        .then(answer => {
          // サーバーからのアンサーを設定
          return webrtcPeerConnection.setRemoteDescription(new RTCSessionDescription(answer));
        })
        .catch(error => {
          console.error('WebRTC接続エラー:', error);
          handleConnectionError('WebRTCシグナリングに失敗しました: ' + error.message);
        });
      
      // ICE候補イベントハンドラ
      webrtcPeerConnection.onicecandidate = function(event) {
        if (event.candidate) {
          // ICE候補をシグナリングサーバーに送信
          const protocol = config.useTLS ? 'https' : 'http';
          const signalingUrl = `${protocol}://${config.serverHost}:${config.serverPort}/rtc-signaling/ice`;
          
          fetch(signalingUrl, {
            method: 'POST',
            headers: {
              'Content-Type': 'application/json'
            },
            body: JSON.stringify({
              type: 'ice-candidate',
              candidate: event.candidate
            })
          }).catch(error => {
            console.error('ICE候補の送信に失敗:', error);
          });
        }
      };
      
    } catch (error) {
      console.error('WebRTC接続の初期化に失敗:', error);
      handleConnectionError('WebRTC接続の初期化に失敗しました: ' + error.message);
    }
  }
  
  /**
   * 認証情報を送信
   */
  function sendAuthInfo() {
    const username = document.getElementById('username').value;
    const password = document.getElementById('password').value;
    
    const command = {
      type: 'Auth',
      username: username,
      password: password
    };
    
    sendCommand(command);
  }
  
  /**
   * WebSocketからのメッセージ処理
   */
  function handleWebSocketMessage(event) {
    try {
      // テキストメッセージはJSONとして解析
      if (typeof event.data === 'string') {
        const response = JSON.parse(event.data);
        handleServerResponse(response);
      } 
      // バイナリデータは画像データとして処理
      else if (event.data instanceof Blob) {
        const reader = new FileReader();
        
        reader.onload = function() {
          const arrayBuffer = this.result;
          processBinaryImageData(arrayBuffer);
        };
        
        reader.readAsArrayBuffer(event.data);
      }
    } catch (error) {
      console.error('メッセージ処理エラー:', error);
    }
  }
  
  /**
   * WebRTCからのメッセージ処理
   */
  function handleWebRTCMessage(event) {
    try {
      // テキストメッセージはJSONとして解析
      if (typeof event.data === 'string') {
        const response = JSON.parse(event.data);
        handleServerResponse(response);
      } 
      // バイナリデータは画像データとして処理
      else if (event.data instanceof ArrayBuffer) {
        processBinaryImageData(event.data);
      }
    } catch (error) {
      console.error('メッセージ処理エラー:', error);
    }
  }
  
  /**
   * サーバーからのレスポンス処理
   */
  function handleServerResponse(response) {
    console.log('レスポンス受信:', response.type);
    
    switch (response.type) {
      case 'AuthResult':
        if (response.success) {
          updateStatus('認証に成功しました');
          // 認証成功後にスクリーンショット要求
          requestScreenshot();
        } else {
          updateStatus('認証に失敗しました: ' + response.message);
        }
        break;
        
      case 'ScreenshotData':
        if (response.data) {
          // データURLの場合
          if (typeof response.data === 'string' && response.data.startsWith('data:')) {
            displayImage(response.data, response.width, response.height);
          }
          // JSON内のバイナリデータはBase64形式で来る場合の処理
          else if (typeof response.data === 'string') {
            const binaryData = atob(response.data);
            const byteArray = new Uint8Array(binaryData.length);
            for (let i = 0; i < binaryData.length; i++) {
              byteArray[i] = binaryData.charCodeAt(i);
            }
            processImageData(byteArray.buffer, response.format, response.width, response.height, response.timestamp);
          }
        }
        break;
        
      case 'Error':
        updateStatus('エラー: ' + response.message);
        break;
        
      case 'Pong':
        calculateLatency(response.original_timestamp, response.server_time);
        break;
        
      case 'CommandResult':
        if (!response.success) {
          updateStatus('コマンド実行エラー: ' + response.message);
        }
        break;
        
      case 'ConnectionStatus':
        updateStatus(response.message);
        if (!response.connected) {
          handleDisconnect();
        }
        break;
        
      case 'SystemInfo':
        updateSystemInfo(response);
        break;
        
      default:
        if (config.debugMode) {
          console.log('未処理のレスポンス:', response);
        }
        break;
    }
  }
  
  /**
   * バイナリ画像データを処理
   */
  function processBinaryImageData(arrayBuffer) {
    // 先頭の4バイトはフォーマット識別子
    const format = new Uint8Array(arrayBuffer.slice(0, 4));
    const formatString = String.fromCharCode.apply(null, format);
    
    // 次の8バイトは画像の幅と高さ（各4バイト）
    const dimensions = new Uint32Array(arrayBuffer.slice(4, 12));
    const width = dimensions[0];
    const height = dimensions[1];
    
    // タイムスタンプ（8バイト）
    const timestamp = new BigUint64Array(arrayBuffer.slice(12, 20))[0];
    
    // 残りが画像データ
    const imageData = arrayBuffer.slice(20);
    
    // フォーマットを識別
    let imageFormat;
    switch(formatString) {
      case 'JPEG':
        imageFormat = 'jpeg';
        break;
      case 'PNG\0':
        imageFormat = 'png';
        break;
      case 'WEBP':
        imageFormat = 'webp';
        break;
      default:
        imageFormat = 'jpeg'; // デフォルト
    }
    
    processImageData(imageData, imageFormat, width, height, Number(timestamp));
  }
  
  /**
   * 画像データを処理して表示
   */
  function processImageData(buffer, format, width, height, timestamp) {
    const blob = new Blob([buffer], { type: `image/${format}` });
    const url = URL.createObjectURL(blob);
    
    const image = new Image();
    image.onload = function() {
      displayImage(image, width, height);
      URL.revokeObjectURL(url);
      
      // FPS計算
      calculateFPS();
      
      // パフォーマンス統計更新
      performanceStats.dataReceived += buffer.byteLength;
      
      // 最新のフレームタイムを更新
      lastFrameTime = performance.now();
    };
    
    image.src = url;
    
    // 画像データをキャッシュ
    latestImageData = {
      buffer,
      format,
      width,
      height,
      timestamp
    };
  }
  
  /**
   * 画像を表示
   */
  function displayImage(image, width, height) {
    // キャンバスのサイズを設定
    if (width && height) {
      canvasElement.width = width;
      canvasElement.height = height;
    }
    
    // 画像を描画
    canvasContext.clearRect(0, 0, canvasElement.width, canvasElement.height);
    canvasContext.drawImage(image, 0, 0, canvasElement.width, canvasElement.height);
  }
  
  /**
   * FPSを計算
   */
  function calculateFPS() {
    const now = performance.now();
    if (lastFrameTime) {
      const delta = now - lastFrameTime;
      if (delta > 0) {
        currentFPS = 1000 / delta;
        
        // 移動平均を使用
        performanceStats.fps = performanceStats.fps * 0.9 + currentFPS * 0.1;
        
        // 表示を更新（1秒ごと）
        fpsDisplay.textContent = `FPS: ${Math.round(performanceStats.fps)}`;
      }
    }
    lastFrameTime = now;
  }
  
  /**
   * レイテンシーを計算
   */
  function calculateLatency(sentTimestamp, serverTime) {
    const now = Date.now();
    const oneWayLatency = (now - sentTimestamp) / 2;
    
    // 統計を更新
    performanceStats.latency = oneWayLatency;
    performanceStats.totalLatency += oneWayLatency;
    performanceStats.latencyCount++;
    performanceStats.avgLatency = performanceStats.totalLatency / performanceStats.latencyCount;
    
    // 表示を更新
    latencyDisplay.textContent = `遅延: ${Math.round(oneWayLatency)}ms`;
  }
  
  /**
   * スクリーンショットを要求
   */
  function requestScreenshot() {
    if (!isConnected || !connection) return;
    
    const command = {
      type: 'RequestScreenshot',
      quality: config.quality,
      width: null,
      height: null,
      monitor: null
    };
    
    sendCommand(command);
  }
  
  /**
   * Pingを送信
   */
  function sendPing() {
    if (!isConnected || !connection) return;
    
    const command = {
      type: 'Ping',
      timestamp: Date.now()
    };
    
    sendCommand(command);
  }
  
  /**
   * コマンドを送信
   */
  function sendCommand(command) {
    if (!isConnected || !connection) return;
    
    try {
      const commandStr = JSON.stringify(command);
      performanceStats.dataSent += commandStr.length;
      
      if (connection.type === 'websocket') {
        connection.socket.send(commandStr);
      } else if (connection.type === 'webrtc') {
        connection.dataChannel.send(commandStr);
      }
      
      if (config.debugMode) {
        console.log('コマンド送信:', command);
      }
    } catch (error) {
      console.error('コマンド送信エラー:', error);
      
      // 接続が切れている場合は再接続
      if (error.name === 'NetworkError' || error.message.includes('closed')) {
        handleDisconnect();
        if (config.autoReconnect) {
          attemptReconnect();
        }
      }
    }
  }
  
  /**
   * 接続エラー処理
   */
  function handleConnectionError(message) {
    console.error('接続エラー:', message);
    
    isConnecting = false;
    updateStatus('エラー: ' + message);
    updateUI();
    
    // 自動再接続
    if (config.autoReconnect && reconnectCount < config.maxReconnectAttempts) {
      attemptReconnect();
    }
  }
  
  /**
   * 切断処理
   */
  function disconnect() {
    if (!isConnected && !isConnecting) return;
    
    // インターバルをクリア
    clearIntervals();
    
    // WebSocketを閉じる
    if (connection && connection.type === 'websocket') {
      connection.socket.close();
    }
    
    // WebRTCを閉じる
    if (connection && connection.type === 'webrtc') {
      connection.dataChannel.close();
      connection.peerConnection.close();
    }
    
    // WebRTC接続をクリア
    webrtcPeerConnection = null;
    
    // 状態をリセット
    isConnected = false;
    isConnecting = false;
    reconnectCount = 0;
    connection = null;
    
    // UI状態を更新
    updateStatus('切断しました');
    updateUI();
  }
  
  /**
   * 切断時の処理
   */
  function handleDisconnect() {
    console.log('接続が切断されました');
    
    // インターバルをクリア
    clearIntervals();
    
    isConnected = false;
    
    // UI状態を更新
    updateStatus('切断されました');
    updateUI();
    
    // 自動再接続
    if (config.autoReconnect && reconnectCount < config.maxReconnectAttempts) {
      attemptReconnect();
    }
  }
  
  /**
   * 再接続を試みる
   */
  function attemptReconnect() {
    reconnectCount++;
    
    const delay = config.reconnectDelay;
    updateStatus(`再接続を試みます (${reconnectCount}/${config.maxReconnectAttempts})... ${delay/1000}秒後`);
    
    setTimeout(() => {
      if (!isConnected && !isConnecting) {
        connect();
      }
    }, delay);
  }
  
  /**
   * 定期的な更新を開始
   */
  function startPeriodicUpdates() {
    // インターバルをクリア
    clearIntervals();
    
    // スクリーンショット更新インターバル
    updateInterval = setInterval(() => {
      if (isConnected) {
        requestScreenshot();
      }
    }, config.updateInterval);
    
    // キープアライブと統計更新のインターバル
    keepAliveInterval = setInterval(() => {
      if (isConnected) {
        sendPing();
        updatePerformanceDisplay();
      }
    }, 5000);
  }
  
  /**
   * インターバルをクリア
   */
  function clearIntervals() {
    if (updateInterval) {
      clearInterval(updateInterval);
      updateInterval = null;
    }
    
    if (keepAliveInterval) {
      clearInterval(keepAliveInterval);
      keepAliveInterval = null;
    }
  }
  
  /**
   * パフォーマンス表示を更新
   */
  function updatePerformanceDisplay() {
    const received = formatDataSize(performanceStats.dataReceived);
    const sent = formatDataSize(performanceStats.dataSent);
    
    document.getElementById('data-stats').textContent = 
      `受信: ${received} | 送信: ${sent}`;
  }
  
  /**
   * 画質変更処理
   */
  function handleQualityChange() {
    config.quality = parseInt(qualitySlider.value);
    document.getElementById('quality-value').textContent = config.quality;
    
    // 設定を保存
    saveConfig();
    
    // 接続中なら画質設定を送信
    if (isConnected) {
      const command = {
        type: 'SetQuality',
        quality: config.quality
      };
      sendCommand(command);
    }
  }
  
  /**
   * システム情報を更新
   */
  function updateSystemInfo(info) {
    const systemInfoElement = document.getElementById('system-info');
    if (!systemInfoElement) return;
    
    // メモリ使用量をフォーマット
    const totalMemory = formatDataSize(info.total_memory);
    const usedMemory = formatDataSize(info.used_memory);
    
    // 稼働時間をフォーマット
    const uptime = formatUptime(info.uptime);
    
    systemInfoElement.innerHTML = `
      <div>CPU: ${info.cpu_model}</div>
      <div>CPU使用率: ${Math.round(info.cpu_usage)}%</div>
      <div>メモリ: ${usedMemory} / ${totalMemory}</div>
      <div>OS: ${info.os_version}</div>
      <div>ホスト名: ${info.hostname}</div>
      <div>稼働時間: ${uptime}</div>
    `;
  }
  
  /**
   * UI状態を更新
   */
  function updateUI() {
    // 接続状態に応じてボタンの有効/無効を切り替え
    connectButton.disabled = isConnected || isConnecting;
    disconnectButton.disabled = !isConnected && !isConnecting;
    
    // 接続フォームの有効/無効を切り替え
    const formElements = document.querySelectorAll('#connection-form input, #connection-form select');
    formElements.forEach(element => {
      element.disabled = isConnected || isConnecting;
    });
    
    // ステータスクラスを設定
    statusElement.className = isConnected ? 'connected' : isConnecting ? 'connecting' : 'disconnected';
  }
  
  /**
   * ステータスメッセージを更新
   */
  function updateStatus(message) {
    statusElement.textContent = message;
    console.log('ステータス:', message);
  }
  
  /**
   * キャンバスのイベントリスナーをセットアップ
   */
  function setupCanvasListeners() {
    if (!canvasElement) return;
    
    canvasElement.addEventListener('mousedown', handleMouseDown);
    canvasElement.addEventListener('mouseup', handleMouseUp);
    canvasElement.addEventListener('mousemove', handleMouseMove);
    canvasElement.addEventListener('wheel', handleMouseWheel);
    canvasElement.addEventListener('contextmenu', event => event.preventDefault());
    
    // キーボードイベント（キャンバスがフォーカスされている場合のみ）
    canvasElement.addEventListener('keydown', handleKeyDown);
    canvasElement.addEventListener('keyup', handleKeyUp);
    
    // タッチイベント
    canvasElement.addEventListener('touchstart', handleTouchStart);
    canvasElement.addEventListener('touchend', handleTouchEnd);
    canvasElement.addEventListener('touchmove', handleTouchMove);
    
    // キャンバスをクリックしたときにフォーカスを設定
    canvasElement.addEventListener('click', function() {
      canvasElement.focus();
    });
    
    // フォーカス状態のスタイル変更
    canvasElement.addEventListener('focus', function() {
      canvasElement.classList.add('focused');
    });
    
    canvasElement.addEventListener('blur', function() {
      canvasElement.classList.remove('focused');
    });
  }
  
  /**
   * マウスダウンイベント処理
   */
  function handleMouseDown(event) {
    if (!isConnected) return;
    
    const button = getMouseButton(event.button);
    const x = getScaledX(event.offsetX);
    const y = getScaledY(event.offsetY);
    
    const command = {
      type: 'MouseDown',
      button,
      x,
      y
    };
    
    sendCommand(command);
    event.preventDefault();
  }
  
  /**
   * マウスアップイベント処理
   */
  function handleMouseUp(event) {
    if (!isConnected) return;
    
    const button = getMouseButton(event.button);
    const x = getScaledX(event.offsetX);
    const y = getScaledY(event.offsetY);
    
    const command = {
      type: 'MouseUp',
      button,
      x,
      y
    };
    
    sendCommand(command);
    event.preventDefault();
  }
  
  /**
   * マウス移動イベント処理
   */
  function handleMouseMove(event) {
    if (!isConnected) return;
    
    // パフォーマンスのため、すべての移動イベントを送信しない
    // 最後の移動から50ms以上経過した場合のみ送信
    const now = Date.now();
    if (lastMouseMoveTime && now - lastMouseMoveTime < 50) {
      return;
    }
    lastMouseMoveTime = now;
    
    const x = getScaledX(event.offsetX);
    const y = getScaledY(event.offsetY);
    
    const command = {
      type: 'MouseMove',
      x,
      y
    };
    
    sendCommand(command);
  }
  
  /**
   * マウスホイールイベント処理
   */
  function handleMouseWheel(event) {
    if (!isConnected) return;
    
    // スクロール量を標準化
    const deltaX = event.deltaX;
    const deltaY = event.deltaY;
    
    const command = {
      type: 'MouseWheel',
      delta_x: deltaX,
      delta_y: deltaY
    };
    
    sendCommand(command);
    event.preventDefault();
  }
  
  /**
   * キーダウンイベント処理
   */
  function handleKeyDown(event) {
    if (!isConnected) return;
    
    const keyCode = mapKeyCode(event.key, event.code);
    const modifiers = getModifiers(event);
    
    const command = {
      type: 'KeyDown',
      key_code: keyCode,
      modifiers
    };
    
    sendCommand(command);
    
    // ブラウザのデフォルト動作を防止（F5リロードなど）
    event.preventDefault();
  }
  
  /**
   * キーアップイベント処理
   */
  function handleKeyUp(event) {
    if (!isConnected) return;
    
    const keyCode = mapKeyCode(event.key, event.code);
    const modifiers = getModifiers(event);
    
    const command = {
      type: 'KeyUp',
      key_code: keyCode,
      modifiers
    };
    
    sendCommand(command);
    event.preventDefault();
  }
  
  /**
   * タッチスタートイベント処理
   */
  function handleTouchStart(event) {
    if (!isConnected) return;
    
    // シングルタッチは左クリック
    if (event.touches.length === 1) {
      const touch = event.touches[0];
      const rect = canvasElement.getBoundingClientRect();
      const x = getScaledX(touch.clientX - rect.left);
      const y = getScaledY(touch.clientY - rect.top);
      
      const command = {
        type: 'MouseDown',
        button: 'left',
        x,
        y
      };
      
      sendCommand(command);
    }
    // ダブルタッチは右クリック
    else if (event.touches.length === 2) {
      const touch = event.touches[0];
      const rect = canvasElement.getBoundingClientRect();
      const x = getScaledX(touch.clientX - rect.left);
      const y = getScaledY(touch.clientY - rect.top);
      
      const command = {
        type: 'MouseDown',
        button: 'right',
        x,
        y
      };
      
      sendCommand(command);
    }
    
    event.preventDefault();
  }
  
  /**
   * タッチエンドイベント処理
   */
  function handleTouchEnd(event) {
    if (!isConnected) return;
    
    // タッチ終了位置を取得
    const rect = canvasElement.getBoundingClientRect();
    let x, y;
    
    // まだ残っているタッチがあれば、その位置を使用
    if (event.touches.length > 0) {
      const touch = event.touches[0];
      x = getScaledX(touch.clientX - rect.left);
      y = getScaledY(touch.clientY - rect.top);
    }
    // なければ最後に変化したタッチの位置を使用
    else if (event.changedTouches.length > 0) {
      const touch = event.changedTouches[0];
      x = getScaledX(touch.clientX - rect.left);
      y = getScaledY(touch.clientY - rect.top);
    }
    
    // シングルタッチが終了した場合は左クリック解放
    if (event.touches.length === 0 && event.changedTouches.length === 1) {
      const command = {
        type: 'MouseUp',
        button: 'left',
        x,
        y
      };
      
      sendCommand(command);
    }
    // ダブルタッチだった場合は右クリック解放
    else if (event.touches.length === 0 && event.changedTouches.length === 2) {
      const command = {
        type: 'MouseUp',
        button: 'right',
        x,
        y
      };
      
      sendCommand(command);
    }
    
    event.preventDefault();
  }
  
  /**
   * タッチ移動イベント処理
   */
  function handleTouchMove(event) {
    if (!isConnected) return;
    
    // パフォーマンスのため、すべての移動イベントを送信しない
    const now = Date.now();
    if (lastTouchMoveTime && now - lastTouchMoveTime < 50) {
      return;
    }
    lastTouchMoveTime = now;
    
    if (event.touches.length === 1) {
      const touch = event.touches[0];
      const rect = canvasElement.getBoundingClientRect();
      const x = getScaledX(touch.clientX - rect.left);
      const y = getScaledY(touch.clientY - rect.top);
      
      const command = {
        type: 'MouseMove',
        x,
        y
      };
      
      sendCommand(command);
    }
    
    event.preventDefault();
  }
  
  /**
   * マウスボタン番号を文字列に変換
   */
  function getMouseButton(button) {
    switch (button) {
      case 0: return 'left';
      case 1: return 'middle';
      case 2: return 'right';
      default: return 'left';
    }
  }
  
  /**
   * 修飾キーの状態を取得
   */
  function getModifiers(event) {
    const modifiers = [];
    if (event.shiftKey) modifiers.push('Shift');
    if (event.ctrlKey) modifiers.push('Control');
    if (event.altKey) modifiers.push('Alt');
    if (event.metaKey) modifiers.push('Meta');
    return modifiers;
  }
  
  /**
   * キーコードのマッピング
   */
  function mapKeyCode(key, code) {
    // このマッピングはサーバー側のコードとの整合性が必要
    // サーバーはVK_* コードを期待している場合の例
    
    // 基本的なASCII文字
    if (key.length === 1 && key.charCodeAt(0) >= 32 && key.charCodeAt(0) <= 126) {
      return key.toUpperCase().charCodeAt(0);
    }
    
    // 特殊キー
    const keyCodes = {
      'Backspace': 8,
      'Tab': 9,
      'Enter': 13,
      'Shift': 16,
      'Control': 17,
      'Alt': 18,
      'CapsLock': 20,
      'Escape': 27,
      'Space': 32,
      'PageUp': 33,
      'PageDown': 34,
      'End': 35,
      'Home': 36,
      'ArrowLeft': 37,
      'ArrowUp': 38,
      'ArrowRight': 39,
      'ArrowDown': 40,
      'Insert': 45,
      'Delete': 46,
      
      // 数字キー
      '0': 48, '1': 49, '2': 50, '3': 51, '4': 52,
      '5': 53, '6': 54, '7': 55, '8': 56, '9': 57,
      
      // アルファベットキー
      'a': 65, 'b': 66, 'c': 67, 'd': 68, 'e': 69,
      'f': 70, 'g': 71, 'h': 72, 'i': 73, 'j': 74,
      'k': 75, 'l': 76, 'm': 77, 'n': 78, 'o': 79,
      'p': 80, 'q': 81, 'r': 82, 's': 83, 't': 84,
      'u': 85, 'v': 86, 'w': 87, 'x': 88, 'y': 89,
      'z': 90,
      
      // テンキー
      'Numpad0': 96, 'Numpad1': 97, 'Numpad2': 98, 'Numpad3': 99, 'Numpad4': 100,
      'Numpad5': 101, 'Numpad6': 102, 'Numpad7': 103, 'Numpad8': 104, 'Numpad9': 105,
      'NumpadMultiply': 106, 'NumpadAdd': 107, 'NumpadSubtract': 109,
      'NumpadDecimal': 110, 'NumpadDivide': 111,
      
      // ファンクションキー
      'F1': 112, 'F2': 113, 'F3': 114, 'F4': 115, 'F5': 116,
      'F6': 117, 'F7': 118, 'F8': 119, 'F9': 120, 'F10': 121,
      'F11': 122, 'F12': 123,
      
      // その他
      'Semicolon': 186, 'Equal': 187, 'Comma': 188, 'Minus': 189,
      'Period': 190, 'Slash': 191, 'Backquote': 192, 'BracketLeft': 219,
      'Backslash': 220, 'BracketRight': 221, 'Quote': 222
    };
    
    if (code in keyCodes) {
      return keyCodes[code];
    }
    if (key in keyCodes) {
      return keyCodes[key];
    }
    
    // キーコードが見つからない場合
    console.warn('未知のキーコード:', key, code);
    return 0;
  }
  
  /**
   * キャンバス内のX座標をサーバー側の座標系に変換
   */
  function getScaledX(x) {
    if (!latestImageData || !latestImageData.width) return x;
    const scaleX = latestImageData.width / canvasElement.width;
    return Math.round(x * scaleX);
  }
  
  /**
   * キャンバス内のY座標をサーバー側の座標系に変換
   */
  function getScaledY(y) {
    if (!latestImageData || !latestImageData.height) return y;
    const scaleY = latestImageData.height / canvasElement.height;
    return Math.round(y * scaleY);
  }
  
  /**
   * WebRTCがサポートされているかを確認
   */
  function isWebRTCSupported() {
    return window.RTCPeerConnection !== undefined &&
           window.RTCSessionDescription !== undefined &&
           window.RTCIceCandidate !== undefined;
  }
  
  /**
   * データサイズをフォーマット
   */
  function formatDataSize(bytes) {
    if (bytes < 1024) {
      return bytes + ' B';
    } else if (bytes < 1024 * 1024) {
      return (bytes / 1024).toFixed(1) + ' KB';
    } else if (bytes < 1024 * 1024 * 1024) {
      return (bytes / (1024 * 1024)).toFixed(1) + ' MB';
    } else {
      return (bytes / (1024 * 1024 * 1024)).toFixed(1) + ' GB';
    }
  }
  
  /**
   * 時間をフォーマット
   */
  function formatTime(seconds) {
    const hours = Math.floor(seconds / 3600);
    const minutes = Math.floor((seconds % 3600) / 60);
    const secs = seconds % 60;
    
    return `${hours}:${minutes.toString().padStart(2, '0')}:${secs.toString().padStart(2, '0')}`;
  }
  
  /**
   * 稼働時間をフォーマット
   */
  function formatUptime(seconds) {
    const days = Math.floor(seconds / 86400);
    const hours = Math.floor((seconds % 86400) / 3600);
    const minutes = Math.floor((seconds % 3600) / 60);
    const secs = seconds % 60;
    
    if (days > 0) {
      return `${days}日 ${hours}時間 ${minutes}分 ${secs}秒`;
    } else if (hours > 0) {
      return `${hours}時間 ${minutes}分 ${secs}秒`;
    } else if (minutes > 0) {
      return `${minutes}分 ${secs}秒`;
    } else {
      return `${secs}秒`;
    }
  }
  
  /**
   * SHA-256ハッシュ関数（単純化したもの、実際の実装では適切なハッシュライブラリを使用すべき）
   */
  function sha256(str) {
    // この実装は単純な例示用です。実際の実装ではCryptoAPIなどを使用すべきです
    return str; // 実際にはハッシュ化すべき
  }
  
  // 初期化を実行
  initialize();
})();