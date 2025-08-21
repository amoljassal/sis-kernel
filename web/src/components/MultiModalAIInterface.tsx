// Multi-Modal AI Interface Component - Phase 6A
// Voice, sketch, and text input for advanced AI integration

import React, { useState, useRef, useEffect } from 'react';
import { advancedAIIntegration } from '../services/advanced-ai-integration';
import type { VoiceInput, SketchInput, NaturalLanguageInput, AIResponse } from '../services/advanced-ai-integration';
import {
  MicrophoneIcon,
  PencilIcon,
  ChatBubbleLeftRightIcon,
  XMarkIcon,
  StopIcon,
  PaperAirplaneIcon,
  CpuChipIcon,
  AcademicCapIcon,
  LightBulbIcon,
  WrenchIcon,
  BugAntIcon
} from '@heroicons/react/24/outline';

interface MultiModalAIInterfaceProps {
  isVisible: boolean;
  onClose: () => void;
  onCodeGenerated?: (code: string, explanation: string) => void;
}

type InputMode = 'text' | 'voice' | 'sketch';
type AIIntent = 'design' | 'debug' | 'optimize' | 'explain' | 'modify';

export const MultiModalAIInterface: React.FC<MultiModalAIInterfaceProps> = ({
  isVisible,
  onClose,
  onCodeGenerated
}) => {
  const [activeMode, setActiveMode] = useState<InputMode>('text');
  const [selectedIntent, setSelectedIntent] = useState<AIIntent>('design');
  const [isProcessing, setIsProcessing] = useState(false);
  const [aiResponse, setAiResponse] = useState<AIResponse | null>(null);
  const [conversationHistory, setConversationHistory] = useState<any[]>([]);
  
  // Text input state
  const [textInput, setTextInput] = useState('');
  
  // Voice input state
  const [isRecording, setIsRecording] = useState(false);
  const [recordingDuration, setRecordingDuration] = useState(0);
  const [audioData, setAudioData] = useState<Uint8Array | null>(null);
  
  // Sketch input state
  const [isDrawing, setIsDrawing] = useState(false);
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const [sketchData, setSketchData] = useState<string | null>(null);
  
  const mediaRecorderRef = useRef<MediaRecorder | null>(null);
  const recordingTimerRef = useRef<NodeJS.Timeout | null>(null);

  useEffect(() => {
    if (isVisible) {
      loadConversationHistory();
    }
  }, [isVisible]);

  const loadConversationHistory = () => {
    const history = advancedAIIntegration.getConversationHistory();
    setConversationHistory(history);
  };

  // =============================================================================
  // TEXT INPUT HANDLING
  // =============================================================================

  const handleTextSubmit = async () => {
    if (!textInput.trim() || isProcessing) return;

    setIsProcessing(true);
    
    try {
      const input: NaturalLanguageInput = {
        text: textInput,
        intent: selectedIntent,
        context: 'multi_modal_interface',
        previousMessages: conversationHistory
      };

      const response = await advancedAIIntegration.processNaturalLanguage(input);
      
      setAiResponse(response);
      setTextInput('');
      
      // Notify parent component if code was generated
      if (response.response.code && onCodeGenerated) {
        onCodeGenerated(response.response.code, response.response.explanation || '');
      }
      
      // Update conversation history
      loadConversationHistory();
      
    } catch (error) {
      console.error('Text processing failed:', error);
      // Show error message to user
    } finally {
      setIsProcessing(false);
    }
  };

  // =============================================================================
  // VOICE INPUT HANDLING
  // =============================================================================

  const startVoiceRecording = async () => {
    try {
      const stream = await navigator.mediaDevices.getUserMedia({ audio: true });
      const mediaRecorder = new MediaRecorder(stream);
      const audioChunks: BlobPart[] = [];

      mediaRecorder.ondataavailable = (event) => {
        audioChunks.push(event.data);
      };

      mediaRecorder.onstop = async () => {
        const audioBlob = new Blob(audioChunks, { type: 'audio/webm' });
        const arrayBuffer = await audioBlob.arrayBuffer();
        const uint8Array = new Uint8Array(arrayBuffer);
        
        setAudioData(uint8Array);
        
        // Stop all tracks to release microphone
        stream.getTracks().forEach(track => track.stop());
      };

      mediaRecorderRef.current = mediaRecorder;
      mediaRecorder.start();
      setIsRecording(true);
      setRecordingDuration(0);

      // Start recording timer
      recordingTimerRef.current = setInterval(() => {
        setRecordingDuration(prev => prev + 1);
      }, 1000);

    } catch (error) {
      console.error('Failed to start recording:', error);
      alert('Microphone access denied or not available');
    }
  };

  const stopVoiceRecording = () => {
    if (mediaRecorderRef.current && isRecording) {
      mediaRecorderRef.current.stop();
      setIsRecording(false);
      
      if (recordingTimerRef.current) {
        clearInterval(recordingTimerRef.current);
        recordingTimerRef.current = null;
      }
    }
  };

  const processVoiceInput = async () => {
    if (!audioData || isProcessing) return;

    setIsProcessing(true);
    
    try {
      const voiceInput: VoiceInput = {
        audioData,
        format: 'webm',
        sampleRate: 44100,
        duration: recordingDuration,
        language: 'english' // Could be made configurable
      };

      const response = await advancedAIIntegration.processVoiceInput(voiceInput);
      
      setAiResponse(response);
      setAudioData(null);
      setRecordingDuration(0);
      
      if (response.response.code && onCodeGenerated) {
        onCodeGenerated(response.response.code, response.response.explanation || '');
      }
      
      loadConversationHistory();
      
    } catch (error) {
      console.error('Voice processing failed:', error);
    } finally {
      setIsProcessing(false);
    }
  };

  // =============================================================================
  // SKETCH INPUT HANDLING
  // =============================================================================

  const startDrawing = (e: React.MouseEvent<HTMLCanvasElement>) => {
    const canvas = canvasRef.current;
    if (!canvas) return;

    const rect = canvas.getBoundingClientRect();
    const x = e.clientX - rect.left;
    const y = e.clientY - rect.top;

    const ctx = canvas.getContext('2d');
    if (ctx) {
      ctx.beginPath();
      ctx.moveTo(x, y);
      setIsDrawing(true);
    }
  };

  const draw = (e: React.MouseEvent<HTMLCanvasElement>) => {
    if (!isDrawing) return;

    const canvas = canvasRef.current;
    if (!canvas) return;

    const rect = canvas.getBoundingClientRect();
    const x = e.clientX - rect.left;
    const y = e.clientY - rect.top;

    const ctx = canvas.getContext('2d');
    if (ctx) {
      ctx.lineWidth = 2;
      ctx.lineCap = 'round';
      ctx.strokeStyle = '#000';
      ctx.lineTo(x, y);
      ctx.stroke();
    }
  };

  const stopDrawing = () => {
    setIsDrawing(false);
    
    // Capture sketch data
    const canvas = canvasRef.current;
    if (canvas) {
      const imageData = canvas.toDataURL('image/png');
      setSketchData(imageData);
    }
  };

  const clearSketch = () => {
    const canvas = canvasRef.current;
    if (canvas) {
      const ctx = canvas.getContext('2d');
      if (ctx) {
        ctx.clearRect(0, 0, canvas.width, canvas.height);
        setSketchData(null);
      }
    }
  };

  const processSketchInput = async () => {
    if (!sketchData || isProcessing) return;

    setIsProcessing(true);
    
    try {
      const sketchInput: SketchInput = {
        imageData: sketchData,
        format: 'png',
        dimensions: {
          width: canvasRef.current?.width || 400,
          height: canvasRef.current?.height || 300
        }
      };

      const response = await advancedAIIntegration.processSketchInput(sketchInput);
      
      setAiResponse(response);
      
      if (response.response.code && onCodeGenerated) {
        onCodeGenerated(response.response.code, response.response.explanation || '');
      }
      
      loadConversationHistory();
      
    } catch (error) {
      console.error('Sketch processing failed:', error);
    } finally {
      setIsProcessing(false);
    }
  };

  // =============================================================================
  // UTILITY FUNCTIONS
  // =============================================================================

  const formatDuration = (seconds: number): string => {
    const mins = Math.floor(seconds / 60);
    const secs = seconds % 60;
    return `${mins}:${secs.toString().padStart(2, '0')}`;
  };

  const getIntentIcon = (intent: AIIntent) => {
    switch (intent) {
      case 'design': return CpuChipIcon;
      case 'debug': return BugAntIcon;
      case 'optimize': return LightBulbIcon;
      case 'explain': return AcademicCapIcon;
      case 'modify': return WrenchIcon;
      default: return CpuChipIcon;
    }
  };

  const getIntentColor = (intent: AIIntent): string => {
    switch (intent) {
      case 'design': return 'text-blue-400';
      case 'debug': return 'text-red-400';
      case 'optimize': return 'text-yellow-400';
      case 'explain': return 'text-green-400';
      case 'modify': return 'text-purple-400';
      default: return 'text-blue-400';
    }
  };

  if (!isVisible) return null;

  return (
    <div className="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50">
      <div className="bg-white rounded-lg shadow-xl w-11/12 h-5/6 max-w-6xl overflow-hidden flex flex-col">
        
        {/* Header */}
        <div className="bg-gradient-to-r from-purple-600 to-indigo-600 text-white p-4 flex justify-between items-center">
          <div className="flex items-center space-x-3">
            <CpuChipIcon className="w-6 h-6" />
            <div>
              <h2 className="text-xl font-bold">Advanced AI Assistant</h2>
              <p className="text-sm opacity-90">Multi-modal hardware design interface</p>
            </div>
          </div>
          <button
            onClick={onClose}
            className="text-white hover:bg-white hover:bg-opacity-20 rounded-full p-2"
          >
            <XMarkIcon className="w-6 h-6" />
          </button>
        </div>

        {/* Main Content */}
        <div className="flex flex-1 overflow-hidden">
          
          {/* Left Panel - Input Methods */}
          <div className="w-1/3 border-r border-gray-200 flex flex-col">
            
            {/* Input Mode Selection */}
            <div className="p-4 border-b border-gray-200">
              <h3 className="font-semibold mb-3">Input Method</h3>
              <div className="flex space-x-2">
                {[
                  { mode: 'text' as InputMode, icon: ChatBubbleLeftRightIcon, label: 'Text' },
                  { mode: 'voice' as InputMode, icon: MicrophoneIcon, label: 'Voice' },
                  { mode: 'sketch' as InputMode, icon: PencilIcon, label: 'Sketch' }
                ].map(({ mode, icon: Icon, label }) => (
                  <button
                    key={mode}
                    onClick={() => setActiveMode(mode)}
                    className={`flex-1 p-3 rounded-lg border-2 transition-colors ${
                      activeMode === mode
                        ? 'border-blue-500 bg-blue-50 text-blue-700'
                        : 'border-gray-200 hover:border-gray-300'
                    }`}
                  >
                    <Icon className="w-5 h-5 mx-auto mb-1" />
                    <div className="text-xs font-medium">{label}</div>
                  </button>
                ))}
              </div>
            </div>

            {/* Intent Selection */}
            <div className="p-4 border-b border-gray-200">
              <h3 className="font-semibold mb-3">AI Intent</h3>
              <div className="grid grid-cols-2 gap-2">
                {[
                  { intent: 'design' as AIIntent, label: 'Design' },
                  { intent: 'debug' as AIIntent, label: 'Debug' },
                  { intent: 'optimize' as AIIntent, label: 'Optimize' },
                  { intent: 'explain' as AIIntent, label: 'Explain' },
                  { intent: 'modify' as AIIntent, label: 'Modify' }
                ].map(({ intent, label }) => {
                  const IconComponent = getIntentIcon(intent);
                  const colorClass = getIntentColor(intent);
                  
                  return (
                    <button
                      key={intent}
                      onClick={() => setSelectedIntent(intent)}
                      className={`p-2 rounded-lg border transition-colors text-left ${
                        selectedIntent === intent
                          ? 'border-blue-500 bg-blue-50'
                          : 'border-gray-200 hover:border-gray-300'
                      }`}
                    >
                      <IconComponent className={`w-4 h-4 ${colorClass} mb-1`} />
                      <div className="text-xs font-medium">{label}</div>
                    </button>
                  );
                })}
              </div>
            </div>

            {/* Input Interface */}
            <div className="flex-1 p-4">
              {activeMode === 'text' && (
                <div className="space-y-4">
                  <textarea
                    value={textInput}
                    onChange={(e) => setTextInput(e.target.value)}
                    placeholder="Describe what you want to design or ask for help..."
                    className="w-full h-32 p-3 border border-gray-300 rounded-lg resize-none focus:ring-2 focus:ring-blue-500 focus:border-transparent"
                    disabled={isProcessing}
                  />
                  <button
                    onClick={handleTextSubmit}
                    disabled={!textInput.trim() || isProcessing}
                    className="w-full btn-primary flex items-center justify-center space-x-2"
                  >
                    <PaperAirplaneIcon className="w-4 h-4" />
                    <span>{isProcessing ? 'Processing...' : 'Send Message'}</span>
                  </button>
                </div>
              )}

              {activeMode === 'voice' && (
                <div className="space-y-4">
                  <div className="text-center">
                    {!isRecording && !audioData && (
                      <button
                        onClick={startVoiceRecording}
                        className="w-20 h-20 bg-red-500 hover:bg-red-600 text-white rounded-full flex items-center justify-center transition-colors"
                        disabled={isProcessing}
                      >
                        <MicrophoneIcon className="w-8 h-8" />
                      </button>
                    )}
                    
                    {isRecording && (
                      <div className="space-y-3">
                        <button
                          onClick={stopVoiceRecording}
                          className="w-20 h-20 bg-red-600 text-white rounded-full flex items-center justify-center animate-pulse"
                        >
                          <StopIcon className="w-8 h-8" />
                        </button>
                        <div className="text-sm text-gray-600">
                          Recording: {formatDuration(recordingDuration)}
                        </div>
                      </div>
                    )}
                    
                    {audioData && (
                      <div className="space-y-3">
                        <div className="text-sm text-green-600">
                          Recording complete: {formatDuration(recordingDuration)}
                        </div>
                        <button
                          onClick={processVoiceInput}
                          disabled={isProcessing}
                          className="w-full btn-primary"
                        >
                          {isProcessing ? 'Processing...' : 'Process Voice Input'}
                        </button>
                        <button
                          onClick={() => setAudioData(null)}
                          className="w-full btn-secondary"
                        >
                          Record Again
                        </button>
                      </div>
                    )}
                  </div>
                  <div className="text-xs text-gray-500 text-center">
                    Click to start recording your hardware design request
                  </div>
                </div>
              )}

              {activeMode === 'sketch' && (
                <div className="space-y-4">
                  <canvas
                    ref={canvasRef}
                    width={300}
                    height={200}
                    className="w-full border border-gray-300 rounded-lg cursor-crosshair bg-white"
                    onMouseDown={startDrawing}
                    onMouseMove={draw}
                    onMouseUp={stopDrawing}
                    onMouseLeave={stopDrawing}
                  />
                  <div className="flex space-x-2">
                    <button
                      onClick={clearSketch}
                      className="flex-1 btn-secondary"
                    >
                      Clear
                    </button>
                    <button
                      onClick={processSketchInput}
                      disabled={!sketchData || isProcessing}
                      className="flex-1 btn-primary"
                    >
                      {isProcessing ? 'Processing...' : 'Analyze Sketch'}
                    </button>
                  </div>
                  <div className="text-xs text-gray-500 text-center">
                    Draw your circuit diagram or hardware concept
                  </div>
                </div>
              )}
            </div>
          </div>

          {/* Right Panel - AI Response */}
          <div className="flex-1 flex flex-col">
            
            {/* Response Header */}
            <div className="p-4 border-b border-gray-200">
              <h3 className="font-semibold">AI Response</h3>
              {aiResponse && (
                <div className="text-xs text-gray-500 mt-1">
                  Confidence: {(aiResponse.response.confidence * 100).toFixed(0)}% | 
                  Processing Time: {aiResponse.response.processingTime}ms | 
                  Model: {aiResponse.metadata.modelUsed}
                </div>
              )}
            </div>

            {/* Response Content */}
            <div className="flex-1 p-4 overflow-y-auto">
              {isProcessing && (
                <div className="flex items-center justify-center h-full">
                  <div className="text-center">
                    <div className="animate-spin rounded-full h-12 w-12 border-b-2 border-blue-500 mx-auto mb-4"></div>
                    <div className="text-gray-600">Processing your request...</div>
                  </div>
                </div>
              )}

              {!isProcessing && !aiResponse && (
                <div className="flex items-center justify-center h-full text-gray-500">
                  <div className="text-center">
                    <CpuChipIcon className="w-16 h-16 mx-auto mb-4 opacity-50" />
                    <div>Ready to help with your hardware design</div>
                    <div className="text-sm mt-2">Choose an input method and start creating</div>
                  </div>
                </div>
              )}

              {!isProcessing && aiResponse && (
                <div className="space-y-4">
                  
                  {aiResponse.response.explanation && (
                    <div>
                      <h4 className="font-semibold mb-2">Explanation</h4>
                      <div className="bg-gray-50 p-3 rounded-lg text-sm whitespace-pre-wrap">
                        {aiResponse.response.explanation}
                      </div>
                    </div>
                  )}

                  {aiResponse.response.code && (
                    <div>
                      <h4 className="font-semibold mb-2">Generated Code</h4>
                      <div className="bg-gray-900 text-green-400 p-4 rounded-lg overflow-x-auto">
                        <pre className="text-sm">
                          <code>{aiResponse.response.code}</code>
                        </pre>
                      </div>
                    </div>
                  )}

                  {aiResponse.response.suggestions && aiResponse.response.suggestions.length > 0 && (
                    <div>
                      <h4 className="font-semibold mb-2">Suggestions</h4>
                      <ul className="space-y-1">
                        {aiResponse.response.suggestions.map((suggestion, index) => (
                          <li key={index} className="text-sm text-gray-600 flex items-start">
                            <span className="w-2 h-2 bg-blue-400 rounded-full mt-2 mr-2 flex-shrink-0"></span>
                            {suggestion}
                          </li>
                        ))}
                      </ul>
                    </div>
                  )}
                </div>
              )}
            </div>

            {/* Conversation History */}
            {conversationHistory.length > 0 && (
              <div className="border-t border-gray-200 p-4 max-h-32 overflow-y-auto">
                <h4 className="font-semibold mb-2 text-sm">Recent Conversation</h4>
                <div className="space-y-1">
                  {conversationHistory.slice(-3).map((message, index) => (
                    <div key={index} className="text-xs">
                      <span className={`font-medium ${
                        message.role === 'user' ? 'text-blue-600' : 'text-green-600'
                      }`}>
                        {message.role === 'user' ? 'You' : 'AI'}:
                      </span>
                      <span className="text-gray-600 ml-2">
                        {message.content.substring(0, 100)}
                        {message.content.length > 100 ? '...' : ''}
                      </span>
                    </div>
                  ))}
                </div>
              </div>
            )}
          </div>
        </div>
      </div>
    </div>
  );
};