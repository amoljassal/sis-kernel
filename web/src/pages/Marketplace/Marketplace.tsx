import React, { useState, useEffect } from 'react';
import { IPBlock } from '../../types/billing';
import { MarketplaceService } from '../../services/marketplace';

const Marketplace: React.FC = () => {
  const [featuredBlocks, setFeaturedBlocks] = useState<IPBlock[]>([]);
  const [popularBlocks, setPopularBlocks] = useState<IPBlock[]>([]);
  const [recentBlocks, setRecentBlocks] = useState<IPBlock[]>([]);
  const [searchQuery, setSearchQuery] = useState('');
  const [searchResults, setSearchResults] = useState<IPBlock[]>([]);
  const [selectedCategory, setSelectedCategory] = useState<string>('');
  const [isLoading, setIsLoading] = useState(true);
  const [activeTab, setActiveTab] = useState<'featured' | 'popular' | 'recent' | 'search'>('featured');

  const marketplaceService = MarketplaceService.getInstance();

  useEffect(() => {
    loadInitialData();
  }, []);

  const loadInitialData = async () => {
    try {
      const [featured, popular, recent] = await Promise.all([
        marketplaceService.getFeaturedIPBlocks(),
        marketplaceService.getPopularIPBlocks(6),
        marketplaceService.getRecentIPBlocks(6)
      ]);
      
      setFeaturedBlocks(featured);
      setPopularBlocks(popular);
      setRecentBlocks(recent);
    } catch (error) {
      console.error('Failed to load marketplace data:', error);
    } finally {
      setIsLoading(false);
    }
  };

  const handleSearch = async () => {
    if (!searchQuery.trim()) return;

    setIsLoading(true);
    setActiveTab('search');
    
    try {
      const results = await marketplaceService.searchIPBlocks(
        searchQuery,
        selectedCategory || undefined
      );
      setSearchResults(results);
    } catch (error) {
      console.error('Search failed:', error);
    } finally {
      setIsLoading(false);
    }
  };

  const handlePurchase = async (ipBlockId: string) => {
    try {
      const purchase = await marketplaceService.purchaseIPBlock('user_123', ipBlockId);
      alert(`Successfully purchased ${purchase.ip_block.name}!`);
    } catch (error) {
      console.error('Purchase failed:', error);
      alert('Purchase failed. Please try again.');
    }
  };

  const IPBlockCard: React.FC<{ block: IPBlock }> = ({ block }) => (
    <div className="card p-4 space-y-3 hover:border-sis-blue-500/50 transition-colors">
      <div className="flex items-start justify-between">
        <div className="flex-1">
          <div className="flex items-center space-x-2 mb-1">
            <h3 className="font-medium text-white">{block.name}</h3>
            {block.verified && <span className="text-xs text-green-400">✓ Verified</span>}
            {block.featured && <span className="text-xs text-yellow-400">⭐ Featured</span>}
          </div>
          <p className="text-sm text-sis-gray-400 line-clamp-2">{block.description}</p>
          <div className="flex items-center space-x-4 mt-2 text-xs text-sis-gray-500">
            <span>by {block.author_name}</span>
            <span>v{block.version}</span>
            <span>{marketplaceService.formatDownloads(block.download_count)} downloads</span>
          </div>
        </div>
        <div className="text-right">
          <div className="text-lg font-bold text-sis-blue-400">
            {marketplaceService.formatPrice(block.price_usd)}
          </div>
          <div className="flex items-center space-x-1 mt-1">
            <span className="text-yellow-400">★</span>
            <span className="text-xs text-sis-gray-400">{block.rating.toFixed(1)} ({block.review_count})</span>
          </div>
        </div>
      </div>

      <div className="flex flex-wrap gap-1">
        {block.tags.slice(0, 4).map(tag => (
          <span key={tag} className="text-xs px-2 py-1 bg-sis-gray-700 rounded-full text-sis-gray-300">
            {tag}
          </span>
        ))}
        {block.tags.length > 4 && (
          <span className="text-xs px-2 py-1 bg-sis-gray-700 rounded-full text-sis-gray-300">
            +{block.tags.length - 4} more
          </span>
        )}
      </div>

      <div className="flex items-center justify-between pt-2 border-t border-sis-gray-700">
        <div className="text-xs text-sis-gray-400">
          <span className="capitalize">{block.category}</span>
        </div>
        <div className="flex space-x-2">
          <button className="btn-secondary text-xs px-3 py-1">
            View Details
          </button>
          {block.price_usd ? (
            <button 
              onClick={() => handlePurchase(block.id)}
              className="btn-primary text-xs px-3 py-1"
            >
              Purchase
            </button>
          ) : (
            <button className="btn-primary text-xs px-3 py-1">
              Download Free
            </button>
          )}
        </div>
      </div>
    </div>
  );

  const getCurrentBlocks = (): IPBlock[] => {
    switch (activeTab) {
      case 'featured': return featuredBlocks;
      case 'popular': return popularBlocks;
      case 'recent': return recentBlocks;
      case 'search': return searchResults;
      default: return featuredBlocks;
    }
  };

  if (isLoading && activeTab !== 'search') {
    return (
      <div className="h-full flex items-center justify-center">
        <div className="text-center">
          <div className="animate-spin w-8 h-8 border-2 border-sis-blue-500 border-t-transparent rounded-full mx-auto mb-4"></div>
          <p className="text-sis-gray-400">Loading marketplace...</p>
        </div>
      </div>
    );
  }

  return (
    <div className="h-full flex flex-col">
      {/* Header */}
      <div className="flex items-center justify-between p-6 border-b border-sis-gray-700">
        <div>
          <h1 className="text-2xl font-bold text-white">IP Marketplace</h1>
          <p className="text-sis-gray-400">Discover and purchase verified IP blocks for your designs</p>
        </div>
        <div className="flex items-center space-x-2">
          <div className="w-2 h-2 bg-green-400 rounded-full"></div>
          <span className="text-sm text-sis-gray-400">Phase 4D: Business Platform</span>
        </div>
      </div>

      {/* Search Bar */}
      <div className="p-6 border-b border-sis-gray-700">
        <div className="flex items-center space-x-4">
          <div className="flex-1">
            <input
              type="text"
              placeholder="Search IP blocks, authors, or categories..."
              value={searchQuery}
              onChange={(e) => setSearchQuery(e.target.value)}
              onKeyDown={(e) => e.key === 'Enter' && handleSearch()}
              className="w-full bg-sis-gray-800 border border-sis-gray-600 rounded-lg px-4 py-2 text-white placeholder-sis-gray-400 focus:outline-none focus:ring-2 focus:ring-sis-blue-500"
            />
          </div>
          <select
            value={selectedCategory}
            onChange={(e) => setSelectedCategory(e.target.value)}
            className="bg-sis-gray-800 border border-sis-gray-600 rounded-lg px-3 py-2 text-white"
          >
            <option value="">All Categories</option>
            <option value="processor">Processors</option>
            <option value="memory">Memory</option>
            <option value="communication">Communication</option>
            <option value="dsp">DSP</option>
            <option value="io">I/O</option>
            <option value="custom">Custom</option>
          </select>
          <button
            onClick={handleSearch}
            className="btn-primary px-6 py-2"
          >
            Search
          </button>
        </div>
      </div>

      {/* Tab Navigation */}
      <div className="flex space-x-1 p-6 pb-0">
        {[
          { key: 'featured', label: 'Featured', icon: '⭐', count: featuredBlocks.length },
          { key: 'popular', label: 'Popular', icon: '🔥', count: popularBlocks.length },
          { key: 'recent', label: 'Recent', icon: '🆕', count: recentBlocks.length }
        ].map(tab => (
          <button
            key={tab.key}
            onClick={() => setActiveTab(tab.key as any)}
            className={`flex items-center space-x-2 px-4 py-2 text-sm font-medium rounded-t-lg transition-colors ${
              activeTab === tab.key
                ? 'bg-sis-gray-800 text-white border-b-2 border-sis-blue-500'
                : 'text-sis-gray-400 hover:text-white hover:bg-sis-gray-800/50'
            }`}
          >
            <span>{tab.icon}</span>
            <span>{tab.label} ({tab.count})</span>
          </button>
        ))}
        {searchResults.length > 0 && (
          <button
            onClick={() => setActiveTab('search')}
            className={`flex items-center space-x-2 px-4 py-2 text-sm font-medium rounded-t-lg transition-colors ${
              activeTab === 'search'
                ? 'bg-sis-gray-800 text-white border-b-2 border-sis-blue-500'
                : 'text-sis-gray-400 hover:text-white hover:bg-sis-gray-800/50'
            }`}
          >
            <span>🔍</span>
            <span>Search Results ({searchResults.length})</span>
          </button>
        )}
      </div>

      {/* Content */}
      <div className="flex-1 p-6 overflow-auto">
        {isLoading ? (
          <div className="text-center py-8">
            <div className="animate-spin w-6 h-6 border-2 border-sis-blue-500 border-t-transparent rounded-full mx-auto mb-2"></div>
            <p className="text-sis-gray-400">Searching...</p>
          </div>
        ) : (
          <div className="grid grid-cols-1 lg:grid-cols-2 xl:grid-cols-3 gap-6">
            {getCurrentBlocks().map(block => (
              <IPBlockCard key={block.id} block={block} />
            ))}
            {getCurrentBlocks().length === 0 && (
              <div className="col-span-full text-center py-12">
                <div className="text-4xl mb-4">🔍</div>
                <h3 className="text-lg font-medium text-white mb-2">No IP blocks found</h3>
                <p className="text-sis-gray-400">
                  {activeTab === 'search' 
                    ? 'Try adjusting your search terms or filters'
                    : 'No blocks available in this category'
                  }
                </p>
              </div>
            )}
          </div>
        )}
      </div>

      {/* Stats Footer */}
      <div className="p-6 border-t border-sis-gray-700">
        <div className="flex items-center justify-between text-sm text-sis-gray-400">
          <div className="flex items-center space-x-6">
            <span>💎 {featuredBlocks.length + popularBlocks.length + recentBlocks.length}+ IP Blocks</span>
            <span>👥 50+ Authors</span>
            <span>📥 100K+ Downloads</span>
            <span>💰 Revenue sharing: 80% author / 20% platform</span>
          </div>
          <div className="flex items-center space-x-2">
            <span className="text-green-400">●</span>
            <span>Marketplace Active</span>
          </div>
        </div>
      </div>
    </div>
  );
}

export default Marketplace