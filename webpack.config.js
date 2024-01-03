const webpack = require('webpack');
const path = require('path');
module.exports = {
  //...
  mode: 'production',
  entry: [
    './static/add-emojis-to-labels.js',
    './static/admin.js',
    './static/admin-modify-voting.js',
    './static/admin-votings.js',
    './static/main.js',
    './static/manage-votings.js',
    './static/user-locator.js'
  ],
  optimization: {
    chunkIds: false,
  },
  output: {
    path: path.resolve(__dirname, 'minified'),
    filename: '[name].js'
  },
  plugins: [
    new webpack.ids.DeterministicChunkIdsPlugin({
      maxLength: 5,
    }),
  ],
};
