// Hero Section
export interface HeroDictionary {
  greeting: string
  name: string
  title: string
  location: string
  tagline: string
  github: string
  contact: string
}

// About Section
export interface AboutDictionary {
  title: string
  learning: {
    title: string
    description: string
    rust: string
    nextjs: string
  }
  expertise: {
    title: string
    description: string
    java: string
  }
}

// Skills Section
export interface SkillsDictionary {
  title: string
  expertise: string
  learning: string
  tools: string
}

// Experience Section
export interface ExperienceItem {
  company: string
  logo: string
  title: string
  dateRange: string
  description: string
}

export interface ExperienceDictionary {
  title: string
  items: ExperienceItem[]
}

// Projects Section
export interface ProjectItem {
  name: string
  description: string
  url: string
  demo: string | null
  stars: number
  forks: number
  topics: string[]
}

export interface ProjectsDictionary {
  title: string
  subtitle: string
  stats: {
    repositories: string
    stars: string
    forks: string
  }
  contributions: string
  viewAll: string
  featured: ProjectItem[]
}

// Testimonials Section
export interface TestimonialItem {
  name: string
  role: string
  company: string
  image: string
  quote: string
}

export interface TestimonialsDictionary {
  title: string
  subtitle: string
  items: TestimonialItem[]
}

// Command Palette
export interface CommandPaletteDictionary {
  placeholder: string
  noResults: string
  navigation: string
  sections: string
  actions: string
  home: string
  imprint: string
  github: string
  email: string
}

// Contact Section
export interface ContactDictionary {
  title: string
  infoTitle: string
  email: string
  github: string
  location: string
  locationValue: string
  downloadResume: string
  imprint: string
}

// Imprint Section
export interface ImprintDictionary {
  title: string
  backHome: string
  infoTitle: string
  contactTitle: string
  responsibleTitle: string
  liabilityContentTitle: string
  liabilityContent: string
  liabilityLinksTitle: string
  liabilityLinks: string
  copyrightTitle: string
  copyright: string
}

// Cookies Banner
export interface CookiesDictionary {
  title: string
  description: string
  acceptAll: string
  rejectAll: string
  customize: string
  essential: string
  essentialDesc: string
  analytics: string
  analyticsDesc: string
  save: string
}

// Main Dictionary Type
export interface Dictionary {
  hero: HeroDictionary
  about: AboutDictionary
  skills: SkillsDictionary
  experience: ExperienceDictionary
  projects: ProjectsDictionary
  testimonials: TestimonialsDictionary
  commandPalette: CommandPaletteDictionary
  contact: ContactDictionary
  imprint: ImprintDictionary
  cookies: CookiesDictionary
}
