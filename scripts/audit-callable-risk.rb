#!/usr/bin/env ruby
# Read-only pinned-model scan. Output is evidence, not an automatic deny policy.
require 'json'
require 'set'

root = File.expand_path(ARGV.fetch(0, '.'))
source = File.expand_path(ARGV.fetch(1, File.join(root, '.local/aws-models')))
report = File.join(root, 'catalog/aws-overlay.report')
owner = /owner|scope|includeshared|includepublic|executableby|isowned|resourceShareType/i
secret = /password|secretstring|secretbinary|privatekey|credentials|(?:^|access|refresh|session|bearer|authorization)token$/i
rows = File.readlines(report, chomp: true).filter_map do |line|
  fields = line.split("\t")
  fields[1, 3] if fields.first == 'operation' && %w[enumerate expand].include?(fields[3])
end
abort 'duplicate callable service/operation' unless rows.map { |r| r.first(2) }.uniq.size == rows.size
models = Dir[File.join(source, 'models/*/service/*/*.json')].group_by do |file|
  file.delete_prefix(File.join(source, 'models/')).split('/').first
end
abort "expected 436 service models, found #{models.size}" unless models.size == 436

def descendants(shapes, target, pattern)
  pending = [[target, '$']]
  visited = Set.new
  hits = []
  until pending.empty?
    id, path = pending.shift
    next unless visited.add?(id)
    shape = shapes.fetch(id, {})
    members = %w[structure union].include?(shape['type']) ? shape.fetch('members', {}) : {}
    members.each do |name, member|
      member_path = "#{path}.#{name}"
      hits << member_path if pattern.match?(name)
      pending << [member.fetch('target'), member_path]
    end
    %w[member value].each do |name|
      edge = shape[name]
      pending << [edge.fetch('target'), "#{path}[]"] if edge
    end
  end
  hits.sort
end

counts = Hash.new { |hash, key| hash[key] = Set.new }
puts "category\tservice\toperation\tclass\tmodel_path\tmember_path\tdocumentation"
rows.group_by(&:first).sort.each do |service, calls|
  files = models.fetch(service)
  abort "ambiguous model for #{service}" unless files.size == 1
  file = files.first
  shapes = JSON.parse(File.read(file)).fetch('shapes')
  calls.each do |_, operation, klass|
    matches = shapes.select { |id, shape| id.end_with?("##{operation}") && shape['type'] == 'operation' }
    abort "missing/ambiguous #{service}/#{operation}" unless matches.size == 1
    op = matches.values.first
    input_id = op.fetch('input').fetch('target')
    input = input_id == 'smithy.api#Unit' ? {} : shapes.fetch(input_id)
    input.fetch('members', {}).each do |name, member|
      next unless name.match?(owner)
      counts['owner_operations'] << [service, operation]
      counts['owner_members'] << [service, operation, name]
      doc = member.fetch('traits', {}).fetch('smithy.api#documentation', '').gsub(/\s+/, ' ')
      puts ['owner', service, operation, klass, file.delete_prefix("#{root}/"), name, doc].join("\t")
    end
    paths = descendants(shapes, op.fetch('output').fetch('target'), secret)
    paths.each do |path|
      counts['secret_operations'] << [service, operation]
      counts['secret_members'] << [service, operation, path]
      puts ['secret', service, operation, klass, file.delete_prefix("#{root}/"), path, 'Name heuristic; pagination/idempotency/reference members require review'].join("\t")
    end
  end
end
warn "callable_operations=#{rows.size} services=#{rows.map(&:first).uniq.size}"
counts.sort.each { |name, values| warn "#{name}=#{values.size}" }
