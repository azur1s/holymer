require "option_parser"

module Trolley
  VERSION = "0.1.0"

  def self.display_help
    puts "trolley " + VERSION
    puts <<-HELP
      USAGE:
          trolley [COMMAND] [OPTION]

      COMMANDS:
          -h, --help        Display this help message
          -v, --version     Display version
      HELP
    exit
  end

  def self.run
    OptionParser.parse do |parser|
      parser.on("-h", "--help") do
        display_help
      end

      parser.on("-v", "--version") do
        puts "trolley " + VERSION
        exit
      end
    end
  end
end

begin
  Trolley.run
end
