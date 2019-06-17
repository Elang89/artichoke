require_relative '../../../spec_helper'
require_relative 'fixtures/classes'

ruby_version_is '2.4' do
  describe 'Enumerator::Lazy#uniq' do
    context 'without block' do
      before :each do
        @lazy = [0, 1, 0, 1].to_enum.lazy.uniq
      end

      it 'returns a lazy enumerator' do
        @lazy.should be_an_instance_of(Enumerator::Lazy)
        @lazy.force.should == [0, 1]
      end

      ruby_bug "#14495", "2.4"..."2.5.2" do
        it 'return same value after rewind' do
          @lazy.force.should == [0, 1]
          @lazy.force.should == [0, 1]
        end
      end

      it 'sets the size to nil' do
        @lazy.size.should == nil
      end
    end

    context 'when yielded with an argument' do
      before :each do
        @lazy = [0, 1, 2, 3].to_enum.lazy.uniq(&:even?)
      end

      it 'returns a lazy enumerator' do
        @lazy.should be_an_instance_of(Enumerator::Lazy)
        @lazy.force.should == [0, 1]
      end

      ruby_bug "#14495", "2.4"..."2.5.2" do
        it 'return same value after rewind' do
          @lazy.force.should == [0, 1]
          @lazy.force.should == [0, 1]
        end
      end

      it 'sets the size to nil' do
        @lazy.size.should == nil
      end
    end

    context 'when yielded with multiple arguments' do
      before :each do
        enum = Object.new.to_enum
        class << enum
          def each
            yield 0, 'foo'
            yield 1, 'FOO'
            yield 2, 'bar'
          end
        end
        @lazy = enum.lazy
      end

      ruby_bug "#14495", "2.4"..."2.5.2" do
        it 'return same value after rewind' do
          enum = @lazy.uniq { |_, label| label.downcase }
          enum.force.should == [[0, 'foo'], [2, 'bar']]
          enum.force.should == [[0, 'foo'], [2, 'bar']]
        end
      end

      it 'returns all yield arguments as an array' do
        @lazy.uniq { |_, label| label.downcase }.force.should == [[0, 'foo'], [2, 'bar']]
      end
    end
  end
end
