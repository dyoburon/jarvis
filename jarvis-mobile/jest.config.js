/** @type {import('jest').Config} */
module.exports = {
  preset: 'jest-expo',
  testMatch: ['**/__tests__/**/*.test.ts', '**/__tests__/**/*.test.tsx'],
  modulePathIgnorePatterns: ['<rootDir>/vendor/'],
  testPathIgnorePatterns: ['/node_modules/'],
  // @noble/* ships ESM; allow Babel to transform it (matches jest-expo pattern + @noble)
  transformIgnorePatterns: [
    '/node_modules/(?!(.pnpm|react-native|@react-native|@react-native-community|expo|@expo|@expo-google-fonts|react-navigation|@react-navigation|@sentry/react-native|native-base|@noble))',
  ],
};
