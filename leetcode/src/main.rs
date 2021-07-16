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

fn main() {}
