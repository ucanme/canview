// script.js for Canview website

// Smooth scrolling navigation
const links = document.querySelectorAll('a[href^="#"]');

links.forEach(link => {
    link.addEventListener('click', function(e) {
        e.preventDefault();
        const targetId = this.getAttribute('href');
        const targetSection = document.querySelector(targetId);
        targetSection.scrollIntoView({ behavior: 'smooth' });
    });
});

// Scroll animations for elements
const animatedElements = document.querySelectorAll('.animate-on-scroll');

const isElementInView = (element) => {
    const rect = element.getBoundingClientRect();
    return rect.top >= 0 && rect.bottom <= window.innerHeight;
};

const handleScrollAnimation = () => {
    animatedElements.forEach(element => {
        if (isElementInView(element)) {
            element.classList.add('visible');
        } else {
            element.classList.remove('visible');
        }
    });
};

window.addEventListener('scroll', handleScrollAnimation);

// Mobile menu toggle
const mobileMenuBtn = document.querySelector('.mobile-menu-button');
const mobileMenu = document.querySelector('.mobile-menu');

mobileMenuBtn.addEventListener('click', () => {
    mobileMenu.classList.toggle('open');
});

// Other interactive features can be added here
