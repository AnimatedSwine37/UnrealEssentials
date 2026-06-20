using System.Collections;
using static UnrealEssentials.Unreal.UnrealMemory;

namespace UnrealEssentials.Unreal;
internal unsafe class UnrealArray
{
    internal struct TArray<T> : IEnumerable<T> where T: unmanaged
    {
        internal T* Values;
        internal int Length;
        internal int Capacity;

        internal void Add(T value)
        {
            if (Length + 1 <= Capacity)
            {
                Values[Length++] = value;
                return;
            }

            Resize(Capacity + 1);
            Values[Length++] = value;
        }

        public IEnumerator<T> GetEnumerator()
        {
            return new TArrayEnumerator<T>(Values, Length);
        }

        internal void Resize(int newCapcity)
        {
            Values = (T*)Mod.Memory.Realloc((nuint)Values, (nuint)(sizeof(T) * newCapcity));
            Capacity = newCapcity;
        }

        IEnumerator IEnumerable.GetEnumerator()
        {
            return GetEnumerator();
        }
    }

    public class TArrayEnumerator<T> : IEnumerator<T> where T: unmanaged
    {
        private T* _values;
        private int _length;
        private int _currentIndex = -1;

        public TArrayEnumerator(T* values, int length)
        {
            _values = values;
            _length = length;
        }

        public T Current => _values[_currentIndex];

        object IEnumerator.Current => Current;

        public bool MoveNext()
        {
            if (_currentIndex + 1 < _length)
            {
                _currentIndex++;
                return true;
            }
            return false;
        }

        public void Reset()
        {
            _currentIndex = 0;
        }

        public void Dispose()
        {
            // We don't really want to dispose anything since _values is coming from Unreal stuff
        }
    }
}
