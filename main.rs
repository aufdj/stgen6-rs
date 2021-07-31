
// stgen6-rs is a rust version of the state table generator stgen6.cpp, created by Matt Mahoney.

/* 
A Counter state represents two numbers, n0 and n1, using 8 bits, and
the following rule:

  Initial state is (0, 0)
  If input is a 0, then n0 is incremented and n1 is reduced.
  If input is a 1, then n1 is incremented and n0 is reduced.

with the exception of values in the range 40-255, which are incremented
probabilistically up to a maximum of 255.  The representable values
are 40, 44, 48, 56, 64, 96, 128, 160, 192, 224, 255. (representable states are multiples of 4, 8, and 32.)

The opposite count is reduced to favor newer data, i.e, if n0 is
incremented then n1 is reduced by the following:

  For 0 <= n1 < 2, n1 is unchanged
  For 2 <= n1 < 25, n1 = n1/2
  for 25 <= n1, n1 = sqrt(n1) + 6, rounded down

There are at most 256 states represented by 8 bits.
For large values of n, an approximate representation is used
with probabilistic increment.  The state table needs the following
mappings from the state s:

  n0
  n1
  next state s00 for input 0 if probabilistic increment fails
  next state s01 for input 0 if probabilistic increment succeeds
  next state s10 for input 1 if probabilistic increment fails
  next state s11 for input 1 if probabilistic increment succeeds
  probability of increment succeeding on input 0 (x 2^32-1)
  probability of increment succeeding on input 1 (x 2^32-1)

The n0 and n1 fields are replaced in the output by get0() and get1()
respectively.  These adjusted counts give higher weights when one
of the counts is small.
*/

use std::collections::BTreeMap;

fn round(n: u32) -> u32 {
         if n < 40  { return n;           }
    else if n < 48  { return n / 4 * 4;   }
    else if n < 64  { return n / 8 * 8;   }
    else if n < 255 { return n / 32 * 32; }
    else            { return 255; }
}
fn inc(n: u32) -> u32 {
    let mut x = 255;
    for i in n..1000 {
        if round(i) > n { 
            x = i; 
            break; 
        }   
    }
    round(x)
}
fn dec(mut n1: u32) -> u32 {
         if n1 < 2  {}
    else if n1 < 25 { n1 /= 2; }
    else            { n1 = ((n1 as f64).sqrt() as u32) + 6; } 
    round(n1)
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Debug)]
struct E {
    n0: u32, n1: u32, 
    s00: u32, s01: u32, s10: u32, s11: u32, 
}
impl E {
    fn new(i: u32, j: u32) -> E {
        E {
            n0: i, n1: j,
            s00: 0, s01: 0, s10: 0, s11: 0, 
        }
    }
    fn print(&self, i: u32) {
        let mut incn0: u32 = inc(self.n0) - self.n0;
        let mut incn1: u32 = inc(self.n1) - self.n1;
        let mut get0: u32 = self.n0 * 2;
        let mut get1: u32 = self.n1 * 2;

             if self.n0 == 0 { get1 *= 2; }
        else if self.n1 == 0 { get0 *= 2; }
        else if self.n0 > self.n1 { 
            get0 /= get1;
            get1 = 1; 
        }
        else if self.n1 > self.n0 {
            get1 /= get0;
            get0 = 1;
        }
        else {
            get0 = 1;
            get1 = 1;
        }

        if incn0 != 0 { incn0 = 0xFFFFFFFF/incn0; } else { incn0 = 0; };
        if incn1 != 0 { incn1 = 0xFFFFFFFF/incn1; } else { incn1 = 0; };
        print!("   [{:4},{:4},{:3},{:3},{:3},{:3},{:10?}u,{:10?}u], // {:3} ({},{})",
                get0, get1, self.s00, self.s01, self.s10, self.s11, 
                incn0, incn1,
                i, self.n0, self.n1);
        println!("");
    }
}

fn main() {
    let mut m: BTreeMap<E, u32> = BTreeMap::new();
    let mut m1: BTreeMap<E, u32> = BTreeMap::new();
    m.insert(E::new(0, 0), 1);
    
    while m.len() != m1.len() {
        m1 = m.clone();
        for (state, _integer) in m1.iter() {
            let e: &E = state;
            m.insert( E::new(    e.n0,  dec(e.n1)), 0 ); 
            m.insert( E::new(inc(e.n0), dec(e.n1)), 0 );
            m.insert( E::new(dec(e.n0),     e.n1 ), 0 ); 
            m.insert( E::new(dec(e.n0), inc(e.n1)), 0 ); 
        }
    }

    let mut v: Vec<E> = Vec::new();

    for (state, _integer) in m.iter() {
        v.push(*state);
    }
    for i in 0..v.len() {
        let mut e: E = v[i];

        let mut n0 = e.n0;
        let mut n1 = dec(e.n1);
        for j in 0..v.len() {
            if v[j].n0 == n0 && v[j].n1 == n1 { e.s00 = j as u32; }
        }

        n0 = inc(n0);
        for j in 0..v.len() {
            if v[j].n0 == n0 && v[j].n1 == n1 { e.s01 = j as u32; }
        }

        n1 = e.n1;
        n0 = dec(e.n0);
        for j in 0..v.len() {
            if v[j].n0 == n0 && v[j].n1 == n1 { e.s10 = j as u32; }
        }

        n1 = inc(n1);
        for j in 0..v.len() {
            if v[j].n0 == n0 && v[j].n1 == n1 { e.s11 = j as u32; }
        }
        v[i] = e;
    }
    println!("//  get0 get1 s00 s01 s10 s11    p(s01)     p(s11)      state  n0,n1");
    println!("//  ---- ---- --- --- --- --- ----------- -----------   ------ -- --");
    for i in 0..v.len() {
        v[i].print(i as u32);
    }
}
