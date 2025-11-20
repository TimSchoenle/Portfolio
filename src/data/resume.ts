import type {
  ResumeEducation,
  ResumeExperience,
  ResumeProject,
  ResumeSkills,
} from '@/types/resume-types'

/**
 * Centralized resume data - single source of truth
 * Used by both PDF generator and website components
 */

export const resumeEducation: readonly ResumeEducation[] = [
  {
    degree: 'B.S. Computer Science',
    institution: 'University of California, Berkeley',
    year: '2019',
  },
]

export const resumeExperience: readonly ResumeExperience[] = [
  {
    achievements: [
      'Led development of microservices architecture serving 1M+ users',
      'Reduced application load time by 60% through optimization',
      'Mentored team of 5 junior developers',
    ],
    company: 'Tech Company Inc.',
    endDate: 'Present',
    location: 'San Francisco, CA',
    startDate: '2021',
    title: 'Senior Software Engineer',
  },
  {
    achievements: [
      'Built real-time messaging system using WebSockets',
      'Implemented CI/CD pipeline reducing deployment time by 80%',
      'Developed RESTful APIs handling 10k requests/minute',
    ],
    company: 'Startup XYZ',
    endDate: '2021',
    location: 'Remote',
    startDate: '2019',
    title: 'Software Engineer',
  },
]

export const resumeProjects: readonly ResumeProject[] = [
  {
    description:
      'Full-stack online store with payment integration and real-time inventory management',
    name: 'E-Commerce Platform',
    technologies: ['React', 'Node.js', 'MongoDB', 'Stripe'],
    url: 'https://github.com/johndoe/ecommerce',
  },
  {
    description:
      'Collaborative project management tool with real-time updates and team features',
    name: 'Task Management App',
    technologies: ['Next.js', 'TypeScript', 'PostgreSQL', 'Prisma'],
  },
]

export const resumeSkills: ResumeSkills = {
  expertise: [
    'JavaScript/TypeScript',
    'React/Next.js',
    'Node.js',
    'Python',
    'PostgreSQL',
  ],
  learning: ['Rust', 'Go', 'Kubernetes'],
  tools: ['Git', 'Docker', 'VS Code', 'AWS', 'CI/CD'],
}

export const resumeSummary: Record<'de' | 'en', string> = {
  de: 'Erfahrener Software-Ingenieur mit über 5 Jahren Expertise in der Entwicklung skalierbarer Webanwendungen. Leidenschaftlich für sauberen Code, moderne Technologien und das Lösen komplexer Probleme.',
  en: 'Experienced software engineer with 5+ years of expertise in building scalable web applications. Passionate about clean code, modern technologies, and solving complex problems.',
} as const

/**
 * Get localized summary
 */
export function getResumeSummary(locale: 'de' | 'en'): string {
  // eslint-disable-next-line security/detect-object-injection
  return resumeSummary[locale]
}
