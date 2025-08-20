import { IPBlock, MarketplacePurchase, MarketplaceReview, RevenueShare, LicenseType } from '../types/billing';

// Mock marketplace service for IP blocks
export class MarketplaceService {
  private static instance: MarketplaceService;
  private ipBlocks: Map<string, IPBlock> = new Map();
  private purchases: Map<string, MarketplacePurchase> = new Map();
  private reviews: Map<string, MarketplaceReview[]> = new Map();

  static getInstance(): MarketplaceService {
    if (!MarketplaceService.instance) {
      MarketplaceService.instance = new MarketplaceService();
      MarketplaceService.instance.initializeBlocks();
    }
    return MarketplaceService.instance;
  }

  private initializeBlocks(): void {
    const mockBlocks: IPBlock[] = [
      {
        id: 'ip_001',
        name: 'High-Performance UART Controller',
        description: 'Advanced UART controller with configurable baud rates, FIFO buffers, and hardware flow control. Optimized for high-throughput communication.',
        category: 'communication',
        author_id: 'author_001',
        author_name: 'TechCorp Solutions',
        version: '2.1.0',
        tags: ['uart', 'communication', 'serial', 'fifo', 'verified'],
        license_type: 'commercial',
        price_usd: 49.99,
        download_count: 1247,
        rating: 4.8,
        review_count: 89,
        created_at: new Date('2024-01-15'),
        updated_at: new Date('2024-02-28'),
        verified: true,
        featured: true,
        compatibility: {
          fpga_vendors: ['xilinx', 'intel', 'lattice'],
          min_logic_cells: 150,
          min_block_ram_kb: 2,
          min_dsp_blocks: 0
        },
        files: {
          verilog_url: '/ip/ip_001/uart_controller.v',
          documentation_url: '/ip/ip_001/documentation.pdf',
          example_url: '/ip/ip_001/examples.zip',
          testbench_url: '/ip/ip_001/testbench.v'
        },
        preview_image_url: '/ip/ip_001/preview.png',
        demo_video_url: '/ip/ip_001/demo.mp4'
      },
      {
        id: 'ip_002',
        name: 'DDR4 Memory Controller',
        description: 'Production-ready DDR4 memory controller supporting up to 3200 MT/s with advanced features like ECC, refresh management, and power optimization.',
        category: 'memory',
        author_id: 'author_002',
        author_name: 'MemoryTech Ltd',
        version: '1.5.2',
        tags: ['ddr4', 'memory', 'controller', 'ecc', 'high-speed'],
        license_type: 'commercial',
        price_usd: 299.99,
        download_count: 543,
        rating: 4.9,
        review_count: 34,
        created_at: new Date('2023-11-20'),
        updated_at: new Date('2024-03-10'),
        verified: true,
        featured: true,
        compatibility: {
          fpga_vendors: ['xilinx', 'intel'],
          min_logic_cells: 5000,
          min_block_ram_kb: 100,
          min_dsp_blocks: 0
        },
        files: {
          verilog_url: '/ip/ip_002/ddr4_controller.v',
          vhdl_url: '/ip/ip_002/ddr4_controller.vhd',
          documentation_url: '/ip/ip_002/documentation.pdf',
          example_url: '/ip/ip_002/examples.zip'
        },
        preview_image_url: '/ip/ip_002/preview.png'
      },
      {
        id: 'ip_003',
        name: 'FFT Accelerator Core',
        description: 'Hardware-optimized Fast Fourier Transform core with configurable point sizes (64 to 4096) and streaming interface for DSP applications.',
        category: 'dsp',
        author_id: 'author_003',
        author_name: 'DSP Innovations',
        version: '3.0.1',
        tags: ['fft', 'dsp', 'streaming', 'accelerator', 'configurable'],
        license_type: 'commercial',
        price_usd: 199.99,
        download_count: 892,
        rating: 4.7,
        review_count: 67,
        created_at: new Date('2023-09-05'),
        updated_at: new Date('2024-01-22'),
        verified: true,
        featured: false,
        compatibility: {
          fpga_vendors: ['xilinx', 'intel', 'lattice'],
          min_logic_cells: 2500,
          min_block_ram_kb: 50,
          min_dsp_blocks: 12
        },
        files: {
          verilog_url: '/ip/ip_003/fft_core.v',
          documentation_url: '/ip/ip_003/documentation.pdf',
          example_url: '/ip/ip_003/examples.zip',
          testbench_url: '/ip/ip_003/testbench.v'
        },
        preview_image_url: '/ip/ip_003/preview.png',
        demo_video_url: '/ip/ip_003/demo.mp4'
      },
      {
        id: 'ip_004',
        name: 'Open-Source AES Encryption Core',
        description: 'Fully open-source AES-256 encryption/decryption core with comprehensive test suite. Perfect for security applications and learning.',
        category: 'custom',
        author_id: 'author_004',
        author_name: 'CryptoFPGA Community',
        version: '1.2.0',
        tags: ['aes', 'encryption', 'security', 'open-source', 'educational'],
        license_type: 'free',
        download_count: 2156,
        rating: 4.6,
        review_count: 145,
        created_at: new Date('2023-06-12'),
        updated_at: new Date('2024-02-15'),
        verified: true,
        featured: false,
        compatibility: {
          fpga_vendors: ['xilinx', 'intel', 'lattice', 'microsemi'],
          min_logic_cells: 800,
          min_block_ram_kb: 10,
          min_dsp_blocks: 0
        },
        files: {
          verilog_url: '/ip/ip_004/aes_core.v',
          vhdl_url: '/ip/ip_004/aes_core.vhd',
          documentation_url: '/ip/ip_004/documentation.pdf',
          example_url: '/ip/ip_004/examples.zip',
          testbench_url: '/ip/ip_004/comprehensive_testbench.v'
        },
        preview_image_url: '/ip/ip_004/preview.png'
      },
      {
        id: 'ip_005',
        name: 'RISC-V Processor Core',
        description: '32-bit RISC-V processor core (RV32I) with optional M and C extensions. Includes debugging interface and comprehensive documentation.',
        category: 'processor',
        author_id: 'author_005',
        author_name: 'RV Core Labs',
        version: '2.3.0',
        tags: ['risc-v', 'processor', 'rv32i', 'embedded', 'debugging'],
        license_type: 'commercial',
        price_usd: 599.99,
        download_count: 324,
        rating: 4.9,
        review_count: 28,
        created_at: new Date('2023-12-08'),
        updated_at: new Date('2024-03-05'),
        verified: true,
        featured: true,
        compatibility: {
          fpga_vendors: ['xilinx', 'intel'],
          min_logic_cells: 8000,
          min_block_ram_kb: 200,
          min_dsp_blocks: 0
        },
        files: {
          verilog_url: '/ip/ip_005/riscv_core.v',
          documentation_url: '/ip/ip_005/documentation.pdf',
          example_url: '/ip/ip_005/examples.zip',
          testbench_url: '/ip/ip_005/testbench.v'
        },
        preview_image_url: '/ip/ip_005/preview.png',
        demo_video_url: '/ip/ip_005/demo.mp4'
      },
      {
        id: 'ip_006',
        name: 'Ethernet MAC Controller',
        description: 'Gigabit Ethernet MAC controller with built-in PHY interface, VLAN support, and comprehensive statistics collection.',
        category: 'communication',
        author_id: 'author_006',
        author_name: 'NetCore Systems',
        version: '1.8.3',
        tags: ['ethernet', 'mac', 'gigabit', 'vlan', 'networking'],
        license_type: 'commercial',
        price_usd: 399.99,
        download_count: 678,
        rating: 4.8,
        review_count: 52,
        created_at: new Date('2023-10-30'),
        updated_at: new Date('2024-02-20'),
        verified: true,
        featured: false,
        compatibility: {
          fpga_vendors: ['xilinx', 'intel'],
          min_logic_cells: 3500,
          min_block_ram_kb: 75,
          min_dsp_blocks: 0
        },
        files: {
          verilog_url: '/ip/ip_006/eth_mac.v',
          documentation_url: '/ip/ip_006/documentation.pdf',
          example_url: '/ip/ip_006/examples.zip'
        },
        preview_image_url: '/ip/ip_006/preview.png'
      }
    ];

    mockBlocks.forEach(block => this.ipBlocks.set(block.id, block));

    // Initialize some mock reviews
    const mockReviews: { [key: string]: MarketplaceReview[] } = {
      'ip_001': [
        {
          id: 'review_001',
          ip_block_id: 'ip_001',
          user_id: 'user_review_001',
          user_name: 'Mike_Engineer',
          rating: 5,
          title: 'Excellent UART implementation',
          content: 'This UART controller exceeded my expectations. The FIFO implementation is solid and the documentation is comprehensive. Used it in a production project without any issues.',
          helpful_count: 23,
          created_at: new Date('2024-02-15'),
          verified_purchase: true
        },
        {
          id: 'review_002',
          ip_block_id: 'ip_001',
          user_id: 'user_review_002',
          user_name: 'Sarah_FPGA',
          rating: 4,
          title: 'Good but could use more examples',
          content: 'The core works great and integrates easily. Would be nice to have more usage examples for different baud rates and configurations.',
          helpful_count: 15,
          created_at: new Date('2024-01-28'),
          verified_purchase: true
        }
      ],
      'ip_002': [
        {
          id: 'review_003',
          ip_block_id: 'ip_002',
          user_id: 'user_review_003',
          user_name: 'MemoryExpert',
          rating: 5,
          title: 'Production-ready DDR4 controller',
          content: 'Deployed this in multiple products. ECC support is flawless and performance is excellent. Worth every penny.',
          helpful_count: 34,
          created_at: new Date('2024-03-01'),
          verified_purchase: true
        }
      ]
    };

    Object.entries(mockReviews).forEach(([ipBlockId, reviews]) => {
      this.reviews.set(ipBlockId, reviews);
    });
  }

  async searchIPBlocks(query: string = '', category?: string, priceRange?: [number, number], tags?: string[]): Promise<IPBlock[]> {
    await new Promise(resolve => setTimeout(resolve, 600));

    let results = Array.from(this.ipBlocks.values());

    // Apply filters
    if (query) {
      const lowerQuery = query.toLowerCase();
      results = results.filter(block => 
        block.name.toLowerCase().includes(lowerQuery) ||
        block.description.toLowerCase().includes(lowerQuery) ||
        block.tags.some(tag => tag.toLowerCase().includes(lowerQuery))
      );
    }

    if (category) {
      results = results.filter(block => block.category === category);
    }

    if (priceRange) {
      results = results.filter(block => {
        const price = block.price_usd || 0;
        return price >= priceRange[0] && price <= priceRange[1];
      });
    }

    if (tags && tags.length > 0) {
      results = results.filter(block =>
        tags.some(tag => block.tags.includes(tag))
      );
    }

    // Sort by relevance (featured first, then by rating and download count)
    results.sort((a, b) => {
      if (a.featured && !b.featured) return -1;
      if (!a.featured && b.featured) return 1;
      
      const scoreA = a.rating * Math.log(a.download_count + 1);
      const scoreB = b.rating * Math.log(b.download_count + 1);
      return scoreB - scoreA;
    });

    return results;
  }

  async getFeaturedIPBlocks(): Promise<IPBlock[]> {
    await new Promise(resolve => setTimeout(resolve, 400));
    
    return Array.from(this.ipBlocks.values())
      .filter(block => block.featured)
      .sort((a, b) => b.rating - a.rating);
  }

  async getPopularIPBlocks(limit: number = 10): Promise<IPBlock[]> {
    await new Promise(resolve => setTimeout(resolve, 500));
    
    return Array.from(this.ipBlocks.values())
      .sort((a, b) => b.download_count - a.download_count)
      .slice(0, limit);
  }

  async getRecentIPBlocks(limit: number = 10): Promise<IPBlock[]> {
    await new Promise(resolve => setTimeout(resolve, 400));
    
    return Array.from(this.ipBlocks.values())
      .sort((a, b) => b.updated_at.getTime() - a.updated_at.getTime())
      .slice(0, limit);
  }

  async getIPBlock(id: string): Promise<IPBlock | null> {
    await new Promise(resolve => setTimeout(resolve, 300));
    return this.ipBlocks.get(id) || null;
  }

  async getIPBlockReviews(ipBlockId: string): Promise<MarketplaceReview[]> {
    await new Promise(resolve => setTimeout(resolve, 400));
    return this.reviews.get(ipBlockId) || [];
  }

  async purchaseIPBlock(userId: string, ipBlockId: string): Promise<MarketplacePurchase> {
    await new Promise(resolve => setTimeout(resolve, 2000)); // Simulate payment processing

    const ipBlock = await this.getIPBlock(ipBlockId);
    if (!ipBlock) {
      throw new Error(`IP Block ${ipBlockId} not found`);
    }

    if (ipBlock.license_type === 'free') {
      throw new Error('Cannot purchase free IP blocks');
    }

    const purchase: MarketplacePurchase = {
      id: `purchase_${Date.now()}`,
      user_id: userId,
      ip_block_id: ipBlockId,
      ip_block: ipBlock,
      price_paid_usd: ipBlock.price_usd || 0,
      license_terms: 'Commercial license with unlimited use in user projects',
      purchased_at: new Date(),
      download_count: 0,
      max_downloads: 10 // Allow 10 downloads
    };

    this.purchases.set(purchase.id, purchase);

    // Update download count
    ipBlock.download_count += 1;
    this.ipBlocks.set(ipBlockId, ipBlock);

    return purchase;
  }

  async getUserPurchases(userId: string): Promise<MarketplacePurchase[]> {
    await new Promise(resolve => setTimeout(resolve, 500));

    return Array.from(this.purchases.values())
      .filter(purchase => purchase.user_id === userId)
      .sort((a, b) => b.purchased_at.getTime() - a.purchased_at.getTime());
  }

  async downloadIPBlock(userId: string, purchaseId: string): Promise<string> {
    await new Promise(resolve => setTimeout(resolve, 1000));

    const purchase = this.purchases.get(purchaseId);
    if (!purchase) {
      throw new Error('Purchase not found');
    }

    if (purchase.user_id !== userId) {
      throw new Error('Unauthorized access to purchase');
    }

    if (purchase.max_downloads && purchase.download_count >= purchase.max_downloads) {
      throw new Error('Download limit exceeded');
    }

    if (purchase.expires_at && purchase.expires_at < new Date()) {
      throw new Error('License has expired');
    }

    // Increment download count
    purchase.download_count += 1;
    this.purchases.set(purchaseId, purchase);

    // Return download URL (would be signed URL in real implementation)
    return `/api/downloads/${purchaseId}/${purchase.ip_block.id}/package.zip`;
  }

  async addReview(userId: string, ipBlockId: string, review: Omit<MarketplaceReview, 'id' | 'user_id' | 'ip_block_id' | 'created_at' | 'helpful_count' | 'verified_purchase'>): Promise<MarketplaceReview> {
    await new Promise(resolve => setTimeout(resolve, 800));

    // Check if user purchased the IP block
    const userPurchases = await this.getUserPurchases(userId);
    const hasPurchased = userPurchases.some(purchase => purchase.ip_block_id === ipBlockId);

    const newReview: MarketplaceReview = {
      id: `review_${Date.now()}`,
      user_id: userId,
      ip_block_id: ipBlockId,
      created_at: new Date(),
      helpful_count: 0,
      verified_purchase: hasPurchased,
      ...review
    };

    const existingReviews = this.reviews.get(ipBlockId) || [];
    existingReviews.push(newReview);
    this.reviews.set(ipBlockId, existingReviews);

    // Update IP block rating and review count
    const ipBlock = this.ipBlocks.get(ipBlockId);
    if (ipBlock) {
      const allReviews = existingReviews;
      ipBlock.review_count = allReviews.length;
      ipBlock.rating = allReviews.reduce((sum, r) => sum + r.rating, 0) / allReviews.length;
      this.ipBlocks.set(ipBlockId, ipBlock);
    }

    return newReview;
  }

  async getMarketplaceStats(): Promise<{
    total_blocks: number;
    total_authors: number;
    total_downloads: number;
    total_revenue: number;
    categories: { [key: string]: number };
    growth_stats: {
      blocks_this_month: number;
      downloads_this_month: number;
      revenue_this_month: number;
    };
  }> {
    await new Promise(resolve => setTimeout(resolve, 600));

    const blocks = Array.from(this.ipBlocks.values());
    const categories: { [key: string]: number } = {};
    
    blocks.forEach(block => {
      categories[block.category] = (categories[block.category] || 0) + 1;
    });

    const totalRevenue = blocks.reduce((sum, block) => {
      return sum + ((block.price_usd || 0) * block.download_count * 0.8); // Assuming 20% platform fee
    }, 0);

    return {
      total_blocks: blocks.length,
      total_authors: new Set(blocks.map(b => b.author_id)).size,
      total_downloads: blocks.reduce((sum, b) => sum + b.download_count, 0),
      total_revenue: totalRevenue,
      categories,
      growth_stats: {
        blocks_this_month: Math.floor(blocks.length * 0.15), // Mock 15% growth
        downloads_this_month: Math.floor(blocks.reduce((sum, b) => sum + b.download_count, 0) * 0.25), // Mock 25% of total
        revenue_this_month: totalRevenue * 0.3 // Mock 30% of total
      }
    };
  }

  async getAuthorRevenue(authorId: string, startDate: Date, endDate: Date): Promise<RevenueShare[]> {
    await new Promise(resolve => setTimeout(resolve, 500));

    // Mock revenue data for author
    const authorBlocks = Array.from(this.ipBlocks.values()).filter(b => b.author_id === authorId);
    
    return authorBlocks.map(block => ({
      id: `revenue_${block.id}`,
      author_id: authorId,
      ip_block_id: block.id,
      period_start: startDate,
      period_end: endDate,
      total_sales: Math.floor(block.download_count * 0.3), // Mock 30% of downloads are sales
      total_revenue_usd: (block.price_usd || 0) * Math.floor(block.download_count * 0.3),
      platform_fee_percent: 20,
      platform_fee_usd: (block.price_usd || 0) * Math.floor(block.download_count * 0.3) * 0.2,
      author_share_usd: (block.price_usd || 0) * Math.floor(block.download_count * 0.3) * 0.8,
      status: 'paid',
      paid_at: new Date(Date.now() - 5 * 24 * 60 * 60 * 1000) // 5 days ago
    }));
  }

  // Utility methods
  formatPrice(price: number | undefined): string {
    if (!price || price === 0) return 'Free';
    return new Intl.NumberFormat('en-US', {
      style: 'currency',
      currency: 'USD'
    }).format(price);
  }

  formatDownloads(count: number): string {
    if (count >= 1000000) {
      return `${(count / 1000000).toFixed(1)}M`;
    } else if (count >= 1000) {
      return `${(count / 1000).toFixed(1)}K`;
    }
    return count.toString();
  }

  getLicenseDescription(type: LicenseType): string {
    switch (type) {
      case 'free':
        return 'Free for personal and commercial use';
      case 'commercial':
        return 'Commercial license required';
      case 'enterprise':
        return 'Enterprise license with support';
      case 'custom':
        return 'Custom licensing terms';
      default:
        return 'License terms available on download';
    }
  }
}