import { Injectable } from '@angular/core';

export interface DiceBearOptions {
  seed?: string;
  backgroundColor?: string[];
  hair?: string[];
  hairColor?: string[];
  skinColor?: string[];
  eyes?: string[];
  eyebrows?: string[];
  mouth?: string[];
  accessories?: string[];
  accessoriesProbability?: number;
  clothing?: string[];
  clothingColor?: string[];
  flip?: boolean;
}

@Injectable({
  providedIn: 'root'
})
export class DiceBearService {
  private readonly baseUrl = 'https://api.dicebear.com/9.x';
  
  // 可用的头像样式 - 支持所有风格
  readonly avatarStyles = [
    { name: 'adventurer', label: '冒险家', description: '卡通风格冒险家头像' },
    { name: 'avataaars', label: 'Avataaars', description: '经典Avataaars风格' },
    { name: 'bottts', label: '机器人', description: '机器人风格头像' },
    { name: 'croodles', label: '涂鸦', description: '手绘涂鸦风格' },
    { name: 'fun-emoji', label: '表情符号', description: '有趣的表情符号' },
    { name: 'lorelei', label: '洛蕾莱', description: '现代简约风格' },
    { name: 'micah', label: '米卡', description: '插画风格头像' },
    { name: 'miniavs', label: '迷你头像', description: '迷你风格头像' },
    { name: 'notionists', label: 'Notion风格', description: 'Notion风格头像' },
    { name: 'open-peeps', label: '开放人物', description: '开放人物风格' },
    { name: 'personas', label: '人物角色', description: '人物角色风格' },
    { name: 'pixel-art', label: '像素艺术', description: '像素艺术风格' }
  ];

  constructor() { }

  /**
   * 生成头像URL
   * @param style 头像样式名称
   * @param options 头像选项
   * @param format 图片格式 (svg, png, jpg, webp, avif)
   * @returns 头像URL
   */
  generateAvatarUrl(style: string, options: DiceBearOptions = {}, format: string = 'svg'): string {
    const params = new URLSearchParams();
    
    // 添加选项参数
    Object.entries(options).forEach(([key, value]) => {
      if (value !== undefined && value !== null) {
        if (Array.isArray(value)) {
          params.append(key, value.join(','));
        } else {
          params.append(key, String(value));
        }
      }
    });

    const queryString = params.toString();
    const url = `${this.baseUrl}/${style}/${format}${queryString ? '?' + queryString : ''}`;
    
    return url;
  }

  /**
   * 生成随机头像选项
   * @param seed 种子值（可选，用于生成一致的头像）
   * @returns 随机头像选项
   */
  generateRandomOptions(seed?: string): DiceBearOptions {
    const options: DiceBearOptions = {};
    
    if (seed) {
      options.seed = seed;
    }

    // 使用DiceBear API官方支持的安全背景色
    const safeBackgrounds = [
      'b6e3f4', 'c0aede', 'd1d4f9', 'ffd5dc', 'ffdfbf',
      'ff9aa2', 'ffb3ba', 'ffdfba', 'ffffba', 'baffc9',
      'bae1ff', 'e6ccff', 'ffccf2', 'ffcccb', 'f0f0f0'
    ];
    options.backgroundColor = [safeBackgrounds[Math.floor(Math.random() * safeBackgrounds.length)]];

    // 简化随机属性，只使用最稳定的选项
    const randomChance = Math.random();
    
    if (randomChance > 0.8) {
      // 20% 概率添加随机属性，使用最稳定的选项
      const safeHairColors = ['black', 'blonde', 'brown', 'red'];
      const safeSkinColors = ['tanned', 'pale', 'light', 'brown'];
      
      if (Math.random() > 0.5) {
        options.hairColor = [safeHairColors[Math.floor(Math.random() * safeHairColors.length)]];
      }
      if (Math.random() > 0.5) {
        options.skinColor = [safeSkinColors[Math.floor(Math.random() * safeSkinColors.length)]];
      }
    }

    return options;
  }

  /**
   * 根据用户名生成头像
   * @param username 用户名
   * @param style 头像样式
   * @returns 头像URL
   */
  generateAvatarForUser(username: string, style: string = 'lorelei'): string {
    // 使用用户名作为seed，确保同一用户总是得到相同的头像
    const seed = username.toLowerCase().replace(/[^a-z0-9]/g, '');
    return `${this.baseUrl}/${style}/svg?seed=${encodeURIComponent(seed)}`;
  }

  /**
   * 生成多个头像选项供用户选择
   * @param count 生成数量
   * @param style 头像样式
   * @returns 头像URL数组
   */
  generateAvatarOptions(count: number = 6, style: string = 'lorelei'): string[] {
    const avatars: string[] = [];
    
    for (let i = 0; i < count; i++) {
      // 使用更简单的方法，只使用seed参数，这是最安全的方式
      const timestamp = Date.now();
      const random = Math.random().toString(36).substring(2, 15);
      const seed = `avatar-${i}-${timestamp}-${random}`;
      
      // 只使用seed参数，避免其他可能不兼容的参数
      const url = `${this.baseUrl}/${style}/svg?seed=${encodeURIComponent(seed)}`;
      avatars.push(url);
    }
    
    return avatars;
  }

  /**
   * 获取头像样式的预览
   * @param style 头像样式
   * @returns 预览头像URL
   */
  getStylePreview(style: string): string {
    return this.generateAvatarUrl(style, { seed: 'preview' });
  }
}
