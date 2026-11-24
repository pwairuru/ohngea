PORTNAME=	ohngea
DISTVERSION=	0.1.0
CATEGORIES=	www net
MASTER_SITES=	https://github.com/ohngea/ohngea/releases/download/v${DISTVERSION}/
DISTNAME=	${PORTNAME}-${DISTVERSION}

MAINTAINER=	ohngea@ohngea.com
COMMENT=	Ohngea PBX - Modern FreeSWITCH control plane (Axum + SeaORM + React)
WWW=		https://github.com/ohngea/ohngea

LICENSE=	MIT
LICENSE_FILE=	${WRKSRC}/LICENSE

RUN_DEPENDS=	postgresql18-server:databases/postgresql18-server \
		nginx:www/nginx \
		freeswitch:net/freeswitch

NO_BUILD=	yes
NO_ARCH=	no

USE_RC_SUBR=	ohngea ohngea_init
USERS=		ohngea
GROUPS=		ohngea

SUB_FILES=	pkg-message
SUB_LIST=	WWWDIR=${WWWDIR} \
		ETCDIR=${ETCDIR} \
		PREFIX=${PREFIX}

OPTIONS_DEFINE=		PGSQL NGINX FREESWITCH DOCS
OPTIONS_DEFAULT=	PGSQL NGINX FREESWITCH
OPTIONS_SUB=		yes

PGSQL_DESC=	PostgreSQL 18 server dependency
NGINX_DESC=	Nginx reverse proxy dependency
FREESWITCH_DESC=	FreeSWITCH media server dependency

PGSQL_RUN_DEPENDS=	postgresql18-server:databases/postgresql18-server
NGINX_RUN_DEPENDS=	nginx:www/nginx
FREESWITCH_RUN_DEPENDS=	freeswitch:net/freeswitch

WWWDIR=		${PREFIX}/www/ohngea
ETCDIR=		${PREFIX}/etc/ohngea

PLIST_SUB+=	WWWDIR=${WWWDIR:S/${PREFIX}\///} \
		ETCDIR=${ETCDIR:S/${PREFIX}\///}

PORTDOCS=	README.md

do-install:
	${MKDIR} ${STAGEDIR}${WWWDIR}/static ${STAGEDIR}${ETCDIR}/nginx ${STAGEDIR}${ETCDIR}/freeswitch
	${INSTALL_PROGRAM} ${WRKSRC}/pbx-api-freebsd-amd64 ${STAGEDIR}${PREFIX}/bin/pbx-api
	(cd ${WRKSRC}/frontend/dist && ${COPYTREE_SHARE} . ${STAGEDIR}${WWWDIR}/static)
	${INSTALL_DATA} ${FILESDIR}/ohngea.env.sample ${STAGEDIR}${ETCDIR}/ohngea.env.sample
	${INSTALL_DATA} ${FILESDIR}/nginx.conf ${STAGEDIR}${ETCDIR}/nginx/nginx.conf.sample
	${INSTALL_DATA} ${FILESDIR}/ohngea.conf ${STAGEDIR}${ETCDIR}/nginx/ohngea.conf.sample
	${INSTALL_DATA} ${FILESDIR}/xml_curl.conf.xml.sample ${STAGEDIR}${ETCDIR}/freeswitch/xml_curl.conf.xml.sample
	${INSTALL_DATA} ${FILESDIR}/xml_cdr.conf.xml.sample ${STAGEDIR}${ETCDIR}/freeswitch/xml_cdr.conf.xml.sample
	${INSTALL_SCRIPT} ${FILESDIR}/ohngea-setup ${STAGEDIR}${PREFIX}/bin/ohngea-setup
	${INSTALL_SCRIPT} ${FILESDIR}/ohngea-url ${STAGEDIR}${PREFIX}/bin/ohngea-url

do-install-DOCS-on:
	${MKDIR} ${STAGEDIR}${DOCSDIR}
	${INSTALL_DATA} ${WRKSRC}/README.md ${STAGEDIR}${DOCSDIR}

.include <bsd.port.mk>
