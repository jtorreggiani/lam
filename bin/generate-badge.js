const fs = require('fs');
const lcovParse = require('lcov-parse');
const { makeBadge } = require('badge-maker');

lcovParse('lcov.info', function(err, data) {
  if (err) {
    console.error(err);
    process.exit(1);
  }
  let totalFound = 0, totalHit = 0;
  data.forEach(file => {
    totalFound += file.lines.found;
    totalHit += file.lines.hit;
  });
  let coverage = totalFound === 0 ? 0 : Math.round(totalHit / totalFound * 100);
  let color = 'red';
  if (coverage >= 90) color = 'brightgreen';
  else if (coverage >= 80) color = 'green';
  else if (coverage >= 70) color = 'yellowgreen';
  else if (coverage >= 60) color = 'yellow';
  else if (coverage >= 50) color = 'orange';

  const format = {
    label: 'coverage',
    message: `${coverage}%`,
    color: color,
  };
  const svg = makeBadge(format);
  fs.writeFileSync('coverage.svg', svg);
  console.log('Coverage badge generated.');
});
