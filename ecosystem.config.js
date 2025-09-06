module.exports = {
  apps: [
    {
      name: 'rusted-quotes-server',
      script: './target/release/server',
      instances: 1,
      autorestart: true,
      watch: false,
      max_memory_restart: '1G',
      env: {
        NODE_ENV: '',
        DATABASE_URL: '',
        INAPPROPRIATE_WORDS: ''
      },
      env_production: {
        NODE_ENV: 'production'
      },
      error_file: './logs/err.log',
      out_file: './logs/out.log',
      log_file: './logs/combined.log',
      time: true
    }
  ]
};