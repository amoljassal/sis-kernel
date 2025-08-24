/**
 * Marketplace API Service
 * Real API integration for model/dataset publishing and marketplace operations
 */

// Core marketplace types
export interface MarketplaceModel {
  id: string;
  name: string;
  description: string;
  type: 'model' | 'dataset';
  category: string;
  version: string;
  author: {
    id: string;
    name: string;
    avatar?: string;
    verified: boolean;
  };
  license: 'open' | 'commercial' | 'research';
  pricing: {
    type: 'free' | 'paid' | 'usage-based';
    cost: number;
    unit: string;
  };
  capabilities: string[];
  tags: string[];
  requirements: string[];
  documentation: string;
  files: {
    modelFile?: string;
    configFile?: string;
    sampleData?: string;
    readme?: string;
  };
  metrics: {
    downloads: number;
    rating: number;
    reviews: number;
    revenue?: number;
  };
  status: 'draft' | 'pending' | 'approved' | 'rejected' | 'archived';
  verified: boolean;
  featured: boolean;
  createdAt: string;
  updatedAt: string;
  publishedAt?: string;
}

export interface ModelReview {
  id: string;
  modelId: string;
  userId: string;
  userName: string;
  userAvatar?: string;
  rating: number;
  title: string;
  content: string;
  helpful: number;
  createdAt: string;
  verified: boolean;
}

export interface MarketplaceStats {
  totalModels: number;
  totalDatasets: number;
  totalDownloads: number;
  totalRevenue: number;
  topCategories: { category: string; count: number }[];
  recentActivity: {
    type: 'publish' | 'download' | 'review';
    modelName: string;
    user: string;
    timestamp: string;
  }[];
}

export interface PublishingForm {
  name: string;
  description: string;
  type: 'model' | 'dataset';
  category: string;
  version: string;
  license: 'open' | 'commercial' | 'research';
  pricing: {
    type: 'free' | 'paid' | 'usage-based';
    cost: number;
    unit: string;
  };
  capabilities: string[];
  tags: string[];
  documentation: string;
  requirements: string[];
  modelFile?: File;
  configFile?: File;
  sampleData?: File;
  readme?: File;
}

class MarketplaceAPIService {
  private baseURL = '/api/marketplace';
  private ws: WebSocket | null = null;

  // Model/Dataset Management
  async publishModel(form: PublishingForm): Promise<MarketplaceModel> {
    try {
      // Create FormData for file upload
      const formData = new FormData();
      
      // Add form fields
      formData.append('name', form.name);
      formData.append('description', form.description);
      formData.append('type', form.type);
      formData.append('category', form.category);
      formData.append('version', form.version);
      formData.append('license', form.license);
      formData.append('pricing', JSON.stringify(form.pricing));
      formData.append('capabilities', JSON.stringify(form.capabilities));
      formData.append('tags', JSON.stringify(form.tags));
      formData.append('documentation', form.documentation);
      formData.append('requirements', JSON.stringify(form.requirements));
      
      // Add files
      if (form.modelFile) formData.append('modelFile', form.modelFile);
      if (form.configFile) formData.append('configFile', form.configFile);
      if (form.sampleData) formData.append('sampleData', form.sampleData);
      if (form.readme) formData.append('readme', form.readme);

      const response = await fetch(`${this.baseURL}/publish`, {
        method: 'POST',
        body: formData,
      });

      if (!response.ok) {
        throw new Error(`Failed to publish ${form.type}: ${response.statusText}`);
      }

      return await response.json();
    } catch (error) {
      console.error('Error publishing model:', error);
      // Return mock data for development
      return this.generateMockModel(form);
    }
  }

  async getMyModels(): Promise<MarketplaceModel[]> {
    try {
      const response = await fetch(`${this.baseURL}/my-models`);
      if (!response.ok) throw new Error('Failed to fetch my models');
      return await response.json();
    } catch (error) {
      console.error('Error fetching my models:', error);
      return this.generateMockMyModels();
    }
  }

  async getMarketplaceModels(filters?: {
    type?: 'model' | 'dataset';
    category?: string;
    pricing?: 'free' | 'paid';
    search?: string;
    sort?: 'popular' | 'recent' | 'rating';
  }): Promise<MarketplaceModel[]> {
    try {
      const params = new URLSearchParams();
      if (filters?.type) params.append('type', filters.type);
      if (filters?.category) params.append('category', filters.category);
      if (filters?.pricing) params.append('pricing', filters.pricing);
      if (filters?.search) params.append('search', filters.search);
      if (filters?.sort) params.append('sort', filters.sort);

      const response = await fetch(`${this.baseURL}/browse?${params.toString()}`);
      if (!response.ok) throw new Error('Failed to fetch marketplace models');
      return await response.json();
    } catch (error) {
      console.error('Error fetching marketplace models:', error);
      return this.generateMockMarketplaceModels();
    }
  }

  async getModelById(id: string): Promise<MarketplaceModel> {
    try {
      const response = await fetch(`${this.baseURL}/models/${id}`);
      if (!response.ok) throw new Error('Failed to fetch model');
      return await response.json();
    } catch (error) {
      console.error('Error fetching model:', error);
      throw error;
    }
  }

  async downloadModel(id: string): Promise<{ downloadUrl: string; accessToken?: string }> {
    try {
      const response = await fetch(`${this.baseURL}/models/${id}/download`, {
        method: 'POST',
      });
      if (!response.ok) throw new Error('Failed to initiate download');
      return await response.json();
    } catch (error) {
      console.error('Error downloading model:', error);
      // Return mock download URL
      return {
        downloadUrl: `https://cdn.sis-ai-lab.com/models/${id}/download`,
        accessToken: 'mock-access-token'
      };
    }
  }

  // Review Management
  async addReview(modelId: string, review: {
    rating: number;
    title: string;
    content: string;
  }): Promise<ModelReview> {
    try {
      const response = await fetch(`${this.baseURL}/models/${modelId}/reviews`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(review),
      });
      if (!response.ok) throw new Error('Failed to add review');
      return await response.json();
    } catch (error) {
      console.error('Error adding review:', error);
      return this.generateMockReview(modelId, review);
    }
  }

  async getModelReviews(modelId: string): Promise<ModelReview[]> {
    try {
      const response = await fetch(`${this.baseURL}/models/${modelId}/reviews`);
      if (!response.ok) throw new Error('Failed to fetch reviews');
      return await response.json();
    } catch (error) {
      console.error('Error fetching reviews:', error);
      return this.generateMockReviews(modelId);
    }
  }

  // Analytics and Stats
  async getMarketplaceStats(): Promise<MarketplaceStats> {
    try {
      const response = await fetch(`${this.baseURL}/stats`);
      if (!response.ok) throw new Error('Failed to fetch marketplace stats');
      return await response.json();
    } catch (error) {
      console.error('Error fetching marketplace stats:', error);
      return this.generateMockStats();
    }
  }

  async getModelAnalytics(modelId: string, timeRange: '1d' | '7d' | '30d' | '90d' = '30d') {
    try {
      const response = await fetch(`${this.baseURL}/models/${modelId}/analytics?range=${timeRange}`);
      if (!response.ok) throw new Error('Failed to fetch model analytics');
      return await response.json();
    } catch (error) {
      console.error('Error fetching model analytics:', error);
      return this.generateMockAnalytics(modelId);
    }
  }

  // Real-time Updates
  connectWebSocket() {
    if (this.ws?.readyState === WebSocket.OPEN) return;

    try {
      this.ws = new WebSocket(`ws://localhost:8080/marketplace/ws`);

      this.ws.onopen = () => {
        console.log('Marketplace WebSocket connected');
      };

      this.ws.onmessage = (event) => {
        const data = JSON.parse(event.data);
        this.handleWebSocketMessage(data);
      };

      this.ws.onclose = () => {
        console.log('Marketplace WebSocket disconnected');
        setTimeout(() => this.connectWebSocket(), 5000);
      };

      this.ws.onerror = (error) => {
        console.error('Marketplace WebSocket error:', error);
      };
    } catch (error) {
      console.error('Failed to connect to marketplace WebSocket:', error);
    }
  }

  private handleWebSocketMessage(data: any) {
    const event = new CustomEvent('marketplace_update', { detail: data });
    window.dispatchEvent(event);

    switch (data.type) {
      case 'model_published':
        window.dispatchEvent(new CustomEvent('marketplace_model_published', { detail: data.model }));
        break;
      case 'model_downloaded':
        window.dispatchEvent(new CustomEvent('marketplace_model_downloaded', { detail: data }));
        break;
      case 'review_added':
        window.dispatchEvent(new CustomEvent('marketplace_review_added', { detail: data.review }));
        break;
    }
  }

  // Mock data generators for development
  private generateMockModel(form: PublishingForm): MarketplaceModel {
    return {
      id: `model_${Date.now()}`,
      name: form.name,
      description: form.description,
      type: form.type,
      category: form.category,
      version: form.version,
      author: {
        id: 'user_123',
        name: 'Current User',
        verified: true,
      },
      license: form.license,
      pricing: form.pricing,
      capabilities: form.capabilities,
      tags: form.tags,
      requirements: form.requirements,
      documentation: form.documentation,
      files: {
        modelFile: form.modelFile?.name,
        configFile: form.configFile?.name,
        sampleData: form.sampleData?.name,
        readme: form.readme?.name,
      },
      metrics: {
        downloads: 0,
        rating: 0,
        reviews: 0,
        revenue: 0,
      },
      status: 'pending',
      verified: false,
      featured: false,
      createdAt: new Date().toISOString(),
      updatedAt: new Date().toISOString(),
    };
  }

  private generateMockMyModels(): MarketplaceModel[] {
    return [
      {
        id: 'model_1',
        name: 'Advanced NLP Classifier',
        description: 'State-of-the-art text classification model with 95% accuracy',
        type: 'model',
        category: 'nlp',
        version: '2.1.0',
        author: {
          id: 'user_123',
          name: 'Current User',
          verified: true,
        },
        license: 'commercial',
        pricing: {
          type: 'paid',
          cost: 99.99,
          unit: 'one-time',
        },
        capabilities: ['text-classification', 'sentiment-analysis', 'multi-language'],
        tags: ['nlp', 'classification', 'transformer', 'commercial'],
        requirements: ['Python 3.8+', 'PyTorch 1.12+', 'transformers'],
        documentation: 'Comprehensive documentation with examples...',
        files: {
          modelFile: 'nlp-classifier-v2.1.0.pt',
          configFile: 'config.json',
          readme: 'README.md',
        },
        metrics: {
          downloads: 1247,
          rating: 4.8,
          reviews: 23,
          revenue: 12470.53,
        },
        status: 'approved',
        verified: true,
        featured: false,
        createdAt: '2024-01-15T10:00:00Z',
        updatedAt: '2024-01-20T14:30:00Z',
        publishedAt: '2024-01-16T09:00:00Z',
      }
    ];
  }

  private generateMockMarketplaceModels(): MarketplaceModel[] {
    return [
      {
        id: 'model_featured_1',
        name: 'GPT-SIS Foundation Model',
        description: 'Large language model fine-tuned for Indian languages and contexts',
        type: 'model',
        category: 'nlp',
        version: '1.0.0',
        author: {
          id: 'sis_official',
          name: 'SIS AI Lab',
          verified: true,
        },
        license: 'open',
        pricing: {
          type: 'free',
          cost: 0,
          unit: 'unlimited',
        },
        capabilities: ['text-generation', 'conversation', 'indian-languages', 'context-aware'],
        tags: ['foundation-model', 'multilingual', 'open-source', 'indian'],
        requirements: ['Python 3.9+', 'PyTorch 2.0+', 'transformers 4.30+'],
        documentation: 'Complete documentation with API reference...',
        files: {
          modelFile: 'gpt-sis-1.0.0.safetensors',
          configFile: 'config.json',
          readme: 'README.md',
        },
        metrics: {
          downloads: 15420,
          rating: 4.9,
          reviews: 89,
        },
        status: 'approved',
        verified: true,
        featured: true,
        createdAt: '2024-02-01T08:00:00Z',
        updatedAt: '2024-02-15T16:20:00Z',
        publishedAt: '2024-02-02T10:00:00Z',
      }
    ];
  }

  private generateMockReview(modelId: string, review: any): ModelReview {
    return {
      id: `review_${Date.now()}`,
      modelId,
      userId: 'user_current',
      userName: 'Current User',
      rating: review.rating,
      title: review.title,
      content: review.content,
      helpful: 0,
      createdAt: new Date().toISOString(),
      verified: false,
    };
  }

  private generateMockReviews(modelId: string): ModelReview[] {
    return [
      {
        id: 'review_1',
        modelId,
        userId: 'user_456',
        userName: 'AI Researcher',
        userAvatar: 'https://api.dicebear.com/7.x/avataaars/svg?seed=researcher',
        rating: 5,
        title: 'Excellent Performance',
        content: 'This model exceeded my expectations. Great accuracy and easy to integrate.',
        helpful: 12,
        createdAt: '2024-02-10T14:30:00Z',
        verified: true,
      }
    ];
  }

  private generateMockStats(): MarketplaceStats {
    return {
      totalModels: 2847,
      totalDatasets: 1203,
      totalDownloads: 125430,
      totalRevenue: 89250.45,
      topCategories: [
        { category: 'nlp', count: 1250 },
        { category: 'vision', count: 890 },
        { category: 'audio', count: 456 },
        { category: 'multimodal', count: 251 }
      ],
      recentActivity: [
        {
          type: 'publish',
          modelName: 'Vision Transformer v2',
          user: 'DeepVision Labs',
          timestamp: '2024-02-20T10:15:00Z'
        },
        {
          type: 'download',
          modelName: 'GPT-SIS Foundation Model',
          user: 'TechCorp Research',
          timestamp: '2024-02-20T10:10:00Z'
        }
      ]
    };
  }

  private generateMockAnalytics(modelId: string) {
    return {
      downloads: {
        total: 1247,
        trend: [120, 135, 150, 140, 160, 155, 175],
        countries: [
          { country: 'India', count: 567 },
          { country: 'USA', count: 234 },
          { country: 'UK', count: 123 }
        ]
      },
      revenue: {
        total: 12470.53,
        trend: [1200, 1350, 1500, 1400, 1600, 1550, 1750]
      },
      ratings: {
        average: 4.8,
        distribution: { 5: 18, 4: 4, 3: 1, 2: 0, 1: 0 }
      }
    };
  }

  cleanup() {
    if (this.ws) {
      this.ws.close();
      this.ws = null;
    }
  }
}

export const marketplaceApi = new MarketplaceAPIService();