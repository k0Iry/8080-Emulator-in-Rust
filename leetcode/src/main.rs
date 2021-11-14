// Definition for singly-linked list.
#[derive(PartialEq, Eq, Clone, Debug)]
pub struct ListNode {
    pub val: i32,
    pub next: Option<Box<ListNode>>,
}

impl ListNode {
    #[inline]
    fn new(val: i32) -> Self {
        ListNode { next: None, val }
    }
}

fn merge_two_lists(l1: Option<Box<ListNode>>, l2: Option<Box<ListNode>>) -> Option<Box<ListNode>> {
    if l1 == None {
        return l2;
    }
    if l2 == None {
        return l1;
    }

    let mut l1 = l1;
    let mut l2 = l2;
    let mut head = Some(Box::new(ListNode::new(-1)));

    // Option implements Deref trait, and so does Box,
    // so Deref coercion is happening here...
    let mut result = head.as_mut().unwrap();

    while l1 != None && l2 != None {
        if l1.as_ref().unwrap().val < l2.as_ref().unwrap().val {
            result.next = l1.clone();
            l1 = l1.unwrap().next;
        } else {
            result.next = l2.clone();
            l2 = l2.unwrap().next;
        }
        result = result.next.as_mut().unwrap()
    }

    if l1 != None {
        result.next = l1;
    } else if l2 != None {
        result.next = l2;
    }

    head.unwrap().next
}

pub fn add_two_numbers(
    l1: Option<Box<ListNode>>,
    l2: Option<Box<ListNode>>,
) -> Option<Box<ListNode>> {
    let mut carry = 0;
    let mut result = Some(Box::new(ListNode::new(-1)));
    let mut result_ref = result.as_mut().unwrap();

    let (mut l1, mut l2) = (l1, l2);

    while l1 != None && l2 != None {
        let value = l1.as_ref().unwrap().val + l2.as_ref().unwrap().val + carry;
        result_ref.next = Some(Box::new(ListNode::new(value % 10)));
        carry = value / 10;
        l1 = l1.unwrap().next;
        l2 = l2.unwrap().next;
        result_ref = result_ref.next.as_mut().unwrap();
    }

    if l2 != None {
        l1 = l2
    }

    while l1 != None {
        let value = l1.as_ref().unwrap().val + carry;
        result_ref.next = Some(Box::new(ListNode::new(value % 10)));
        carry = value / 10;
        l1 = l1.unwrap().next;
        result_ref = result_ref.next.as_mut().unwrap();
    }

    if carry > 0 {
        result_ref.next = Some(Box::new(ListNode::new(1)));
    }

    result.unwrap().next
}

pub fn find_min(nums: Vec<i32>) -> i32 {
    let size = nums.len();
    if nums[0] <= nums[size - 1] {
        return nums[0];
    }
    if nums[size - 1] < nums[size - 2] {
        return nums[size - 1];
    }

    let mut l = 0;
    let mut r = size - 1;
    while l <= r {
        let mid = l + (r - l) / 2;
        if nums[mid] < nums[mid - 1] {
            return nums[mid];
        } else if nums[mid] > nums[0] {
            // left side
            l = mid
        } else {
            // right side
            r = mid
        }
    }

    nums[l]
}

pub fn find_min_dup(nums: Vec<i32>) -> i32 {
    let mut l = 0;
    let mut r = nums.len() - 1;

    while l < r {
        let mid = l + (r - l) / 2;
        if nums[mid] < nums[r] {
            // left side, mid "might" be the target we are looking for, so we don't do "-1" aggressively
            r = mid
        } else if nums[mid] > nums[r] {
            // right side, and we can be sure that mid is not possibly the target
            l = mid + 1
        } else {
            r -= 1
        }
    }

    nums[l]
}

pub fn two_sum(numbers: Vec<i32>, target: i32) -> Vec<i32> {
    let mut l = 0;
    let mut r = numbers.len() - 1;
    let mut result: Vec<i32> = vec![];

    while l < r {
        let mid = l + (r - l) / 2;
        let sum = numbers[l] + numbers[r];
        if sum > target {
            if numbers[l] + numbers[mid] < target {
                r -= 1
            } else {
                r = mid
            }
        } else if sum < target {
            if numbers[r] + numbers[mid] > target {
                l += 1
            } else {
                l = mid
            }
        } else {
            result.push((l + 1) as i32);
            result.push((r + 1) as i32);
            break;
        }
    }

    result
}

pub fn find_duplicate(nums: Vec<i32>) -> i32 {
    let mut slow = nums[0] as i32;
    let mut fast = nums[0] as i32;

    loop {
        slow = nums[slow as usize];
        fast = nums[nums[fast as usize] as usize];
        if slow == fast {
            break;
        }
    }

    let mut result = nums[0];
    // move slow
    while slow != result {
        slow = nums[slow as usize];
        result = nums[result as usize];
    }

    result
}

use libc::INT_MAX;
use libc::INT_MIN;
use std::cmp;

pub fn find_median_sorted_arrays(nums1: Vec<i32>, nums2: Vec<i32>) -> f64 {
    let len1 = nums1.len();
    let len2 = nums2.len();
    if len1 > len2 {
        return find_median_sorted_arrays(nums2, nums1);
    }

    let mut l = 0;
    let mut r = len1;

    while l <= r {
        let mid1 = l + (r - l) / 2;
        let mid2 = (len1 + len2) / 2 - mid1;

        let l1 = if mid1 == 0 { INT_MIN } else { nums1[mid1 - 1] };
        let r1 = if mid1 == 0 { INT_MIN } else { nums1[mid1] };

        let l2 = if mid2 == 0 { INT_MAX } else { nums2[mid2 - 1] };
        let r2 = if mid2 == 0 { INT_MAX } else { nums2[mid2] };
        if l1 > r2 {
            r = mid1 - 1
        } else if l2 > r1 {
            l = mid1 + 1
        } else {
            if (len1 + len2) % 2 == 1 {
                return cmp::min(r1, r2) as f64;
            } else {
                return (cmp::max(l1, l2) + cmp::min(r1, r2)) as f64 / 2.0;
            }
        }
    }

    0.0
}

use std::cell::RefCell;
use std::rc::Rc;
#[derive(Debug, PartialEq, Eq)]
pub struct TreeNode {
    pub val: i32,
    pub left: Option<Rc<RefCell<TreeNode>>>,
    pub right: Option<Rc<RefCell<TreeNode>>>,
}

impl TreeNode {
    #[inline]
    pub fn new(val: i32) -> Self {
        TreeNode {
            val,
            left: None,
            right: None,
        }
    }
}

fn dfs(
    root: &Option<Rc<RefCell<TreeNode>>>,
    left_most: bool,
    right_most: bool,
    results: &mut Vec<i32>,
) {
    if *root == None {
        return;
    }
    let root_ref = root.as_ref().unwrap().borrow();
    if left_most || (root_ref.left == None && root_ref.right == None) {
        results.push(root_ref.val)
    }

    dfs(
        &root_ref.left,
        left_most,
        if root_ref.right == None {
            right_most
        } else {
            false
        },
        results,
    );
    dfs(
        &root_ref.right,
        if root_ref.left == None {
            left_most
        } else {
            false
        },
        right_most,
        results,
    );

    if right_most && (root_ref.left != None || root_ref.right != None) {
        results.push(root_ref.val)
    }
}

pub fn boundary_of_binary_tree(root: Option<Rc<RefCell<TreeNode>>>) -> Vec<i32> {
    let mut results: Vec<i32> = Vec::new();
    let root_ref = root.as_ref().unwrap().borrow();
    results.push(root_ref.val);
    dfs(&root_ref.left, true, false, &mut results);
    dfs(&root_ref.right, false, true, &mut results);

    results
}

pub fn backtrack(results: &mut Vec<String>, current: &mut String, open: i32, close: i32, max: i32) {
    if current.len() as i32 == max * 2 {
        results.push(current.clone());
        return;
    }

    if open < max {
        current.push_str("(");
        backtrack(results, current, open + 1, close, max);
        current.pop().unwrap();
    }
    if close < open {
        current.push_str(")");
        backtrack(results, current, open, close + 1, max);
        current.pop().unwrap();
    }
}

pub fn generate_parenthesis(n: i32) -> Vec<String> {
    let mut results = vec![];
    let mut current = String::new();
    backtrack(results.as_mut(), &mut current, 0, 0, n);

    results
}

use std::collections::{HashMap, HashSet, VecDeque};
use std::hash::{Hash, Hasher};

struct point {
    x: i32,
    y: i32,
}

impl point {
    fn new(x: i32, y: i32) -> Self {
        point { x, y }
    }

    fn add(&self, another: &Self) -> Self {
        point {
            x: self.x + another.x,
            y: self.y + another.y,
        }
    }

    fn equal(&self, another: &Self) -> bool {
        self.x == another.x && self.y == another.y
    }
}

impl Clone for point {
    fn clone(&self) -> Self {
        point::new(self.x, self.y)
    }
}

impl PartialEq for point {
    fn eq(&self, other: &Self) -> bool {
        self.x == other.x && self.y == other.y
    }
}

impl Eq for point {}

impl Hash for point {
    fn hash<H>(&self, state: &mut H)
    where
        H: Hasher,
    {
        self.x.hash(state);
        self.y.hash(state);
    }
}

pub fn dfs_min_knight_moves(x: i32, y: i32, save: &mut HashMap<String, i32>) -> i32 {
    let mut key = x.to_string();
    key.push_str(",");
    key.push_str(&y.to_string()[..]);
    if save.get(&key) != None {
        return save[&key];
    }
    if x == 0 && y == 0 {
        return 0;
    }
    if x + y == 2 {
        return 2;
    }

    let val = cmp::min(
        dfs_min_knight_moves((x - 1).abs(), (y - 2).abs(), save),
        dfs_min_knight_moves((x - 2).abs(), (y - 1).abs(), save),
    ) + 1;
    save.insert(key, val);
    val
}

pub fn min_knight_moves(x: i32, y: i32) -> i32 {
    let mut steps = 0;
    let moves = [
        point::new(2, 1),
        point::new(2, -1),
        point::new(1, 2),
        point::new(1, -2),
        point::new(-2, 1),
        point::new(-2, -1),
        point::new(-1, 2),
        point::new(-1, -2),
    ];
    let start = point::new(0, 0);
    let target = point::new(x, y);
    let mut queue: VecDeque<point> = VecDeque::new();
    let mut visited: HashSet<point> = HashSet::new();
    queue.push_back(start);
    while !queue.is_empty() {
        let queue_size = queue.len();
        for _ in 0..queue_size {
            let current = queue.pop_front().unwrap();
            if visited.contains(&current) {
                continue;
            }
            visited.insert(current.clone());
            if target.equal(&current) {
                return steps;
            } else {
                for next_step in &moves {
                    let new_move = current.add(next_step);
                    if !visited.contains(&new_move) {
                        queue.push_back(new_move)
                    }
                }
            }
        }

        steps += 1
    }
    steps
}

struct WordDistance {
    wordsDict: Vec<String>,
}

/**
 * `&self` means the method takes an immutable reference.
 * If you need a mutable reference, change it to `&mut self` instead.
 */
impl WordDistance {
    fn new(wordsDict: Vec<String>) -> Self {
        WordDistance { wordsDict }
    }
    fn shortest(&self, word1: String, word2: String) -> i32 {
        let mut last_access: i32 = -1;

        let number_of_words = self.wordsDict.len();
        let mut answer = number_of_words as i32;

        for index in 0..number_of_words {
            let string = &self.wordsDict[index];
            if string == &word1 || string == &word2 {
                if last_access != -1 && string != &self.wordsDict[last_access as usize] {
                    answer = cmp::min(answer, (last_access - index as i32).abs())
                }
                last_access = index as i32
            }
        }

        answer
    }
}

pub fn expressive_words(s: String, words: Vec<String>) -> i32 {
    let mut result = 0;
    let target_length = s.len();
    for word in words {
        let word_length = word.len();
        if word_length > target_length {
            continue;
        }
        let mut start1 = 0;
        let mut start2 = 0;
        let matched = loop {
            if start1 < target_length && start2 < word_length {
                if s.as_bytes()[start1] != word.as_bytes()[start2] {
                    break false;
                } else {
                    let mut count1 = 0;
                    let mut count2 = 0;
                    while start1 < target_length - 1
                        && s.as_bytes()[start1] == s.as_bytes()[start1 + 1]
                    {
                        start1 += 1;
                        count1 += 1;
                    }
                    start1 += 1;
                    while start2 < word_length - 1
                        && word.as_bytes()[start2] == word.as_bytes()[start2 + 1]
                    {
                        start2 += 1;
                        count2 += 1;
                    }
                    start2 += 1;
                    if count2 > count1 || (count1 > count2 && count1 < 2) {
                        break false;
                    }
                }
            } else {
                break true;
            }
        };

        if matched && start1 == target_length && start2 == word_length {
            result += 1
        }
    }

    result
}

fn main() {}
