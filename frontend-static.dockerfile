FROM nginx:latest

COPY ./ui/ /usr/share/nginx/html

EXPOSE 80
