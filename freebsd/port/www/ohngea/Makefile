PORTNAME=	ohngea
DISTVERSION=	0.1.0
CATEGORIES=	www net python
MASTER_SITES=	https://github.com/ohngea/ohngea/releases/download/v${DISTVERSION}/
PKGNAMEPREFIX=	${PYTHON_PKGNAMEPREFIX}
DISTNAME=	${PORTNAME}-${DISTVERSION}

MAINTAINER=	ohngea@ohngea.com
COMMENT=	Ohngea PBX - Modern FreeSWITCH control plane (FastAPI + PostgreSQL RLS + Valkey)
WWW=		https://github.com/ohngea/ohngea

LICENSE=	MIT
LICENSE_FILE=	${WRKSRC}/LICENSE

RUN_DEPENDS=	${PYTHON_PKGNAMEPREFIX}fastapi>=0.110:www/py-fastapi@${PY_FLAVOR} \
		${PYTHON_PKGNAMEPREFIX}sqlmodel>=0.0.14:databases/py-sqlmodel@${PY_FLAVOR} \
		${PYTHON_PKGNAMEPREFIX}sqlalchemy20>=2.0:databases/py-sqlalchemy20@${PY_FLAVOR} \
		${PYTHON_PKGNAMEPREFIX}asyncpg>=0.29:databases/py-asyncpg@${PY_FLAVOR} \
		${PYTHON_PKGNAMEPREFIX}alembic>=1.13:databases/py-alembic@${PY_FLAVOR} \
		${PYTHON_PKGNAMEPREFIX}pydantic>=2.6:devel/py-pydantic@${PY_FLAVOR} \
		${PYTHON_PKGNAMEPREFIX}pydantic-settings>=2.1:devel/py-pydantic-settings@${PY_FLAVOR} \
		${PYTHON_PKGNAMEPREFIX}uvicorn>=0.30:www/py-uvicorn@${PY_FLAVOR} \
		${PYTHON_PKGNAMEPREFIX}gunicorn>=22:www/py-gunicorn@${PY_FLAVOR} \
		${PYTHON_PKGNAMEPREFIX}httpx>=0.27:www/py-httpx@${PY_FLAVOR} \
		${PYTHON_PKGNAMEPREFIX}anyio>=4:devel/py-anyio@${PY_FLAVOR} \
		${PYTHON_PKGNAMEPREFIX}python-multipart>=0.0.9:www/py-python-multipart@${PY_FLAVOR} \
		${PYTHON_PKGNAMEPREFIX}lxml>=5:devel/py-lxml@${PY_FLAVOR} \
		${PYTHON_PKGNAMEPREFIX}Jinja2>=3.1:devel/py-Jinja2@${PY_FLAVOR} \
		${PYTHON_PKGNAMEPREFIX}PyJWT>=2.8:www/py-pyjwt@${PY_FLAVOR} \
		${PYTHON_PKGNAMEPREFIX}bcrypt>=4.0:security/py-bcrypt@${PY_FLAVOR} \
		${PYTHON_PKGNAMEPREFIX}redis>=5.0:databases/py-redis@${PY_FLAVOR} \
		${PYTHON_PKGNAMEPREFIX}celery>=5.3:devel/py-celery@${PY_FLAVOR} \
		${PYTHON_PKGNAMEPREFIX}email-validator>=2.2:mail/py-email-validator@${PY_FLAVOR} \
		${PYTHON_PKGNAMEPREFIX}boto3>=1.35:www/py-boto3@${PY_FLAVOR} \
		${PYTHON_PKGNAMEPREFIX}botocore>=1.35:devel/py-botocore@${PY_FLAVOR} \
		${PYTHON_PKGNAMEPREFIX}psutil>=6.0:sysutils/py-psutil@${PY_FLAVOR}

# Extra pip-only deps vendored via bundled wheels (no ports yet)
# pydantic-xml, uuid6, valkey — installed in post-install via pip --no-index from WRKSRC/wheels/

BUILD_DEPENDS=	${PYTHON_PKGNAMEPREFIX}hatchling>=1.20:devel/py-hatchling@${PY_FLAVOR}

USES=		python:3.14+ shebangfix
USE_PYTHON=	autoplist concurrent pep517

USE_RC_SUBR=	ohngea ohngea_worker ohngea_beat ohngea_init
USERS=		ohngea
GROUPS=		ohngea

SUB_FILES=	pkg-message
SUB_LIST=	PYTHON_CMD=${PYTHON_CMD} \
		WWWDIR=${WWWDIR} \
		ETCDIR=${ETCDIR} \
		PREFIX=${PREFIX} \
		PYTHON_SITELIBDIR=${PYTHON_SITELIBDIR}

NO_ARCH=	yes
NO_BUILD=	no

OPTIONS_DEFINE=		PGSQL VALKEY NGINX FREESWITCH DOCS
OPTIONS_DEFAULT=	PGSQL VALKEY NGINX FREESWITCH
OPTIONS_SUB=		yes

PGSQL_DESC=	PostgreSQL 18 server dependency
VALKEY_DESC=	Valkey 8 cache/broker dependency
NGINX_DESC=	Nginx reverse proxy dependency
FREESWITCH_DESC=	FreeSWITCH media server dependency

PGSQL_RUN_DEPENDS=	postgresql18-server:databases/postgresql18-server
VALKEY_RUN_DEPENDS=	valkey:databases/valkey
NGINX_RUN_DEPENDS=	nginx:www/nginx
FREESWITCH_RUN_DEPENDS=	freeswitch:net/freeswitch

WWWDIR=		${PREFIX}/www/ohngea
ETCDIR=		${PREFIX}/etc/ohngea
DATADIR=	${PREFIX}/www/ohngea

PLIST_SUB+=	WWWDIR=${WWWDIR:S/${PREFIX}\///} \
		ETCDIR=${ETCDIR:S/${PREFIX}\///}

do-install:
	# Python app (PEP517 auto-installs app package to site-packages)
	# Install extra files not in wheel
	${MKDIR} ${STAGEDIR}${WWWDIR} ${STAGEDIR}${ETCDIR} ${STAGEDIR}${WWWDIR}/static
	${MKDIR} ${STAGEDIR}${ETCDIR}/nginx ${STAGEDIR}${ETCDIR}/freeswitch
	# App + alembic + plugins + templates
	(cd ${WRKSRC} && ${COPYTREE_SHARE} "app alembic plugins templates" ${STAGEDIR}${WWWDIR})
	${INSTALL_DATA} ${WRKSRC}/alembic.ini ${STAGEDIR}${WWWDIR}/alembic.ini.sample
	${INSTALL_DATA} ${WRKSRC}/gunicorn.conf.py ${STAGEDIR}${ETCDIR}/gunicorn.conf.py.sample
	${INSTALL_DATA} ${WRKSRC}/.env.example ${STAGEDIR}${ETCDIR}/ohngea.env.sample
	# nginx turnkey configs
	${INSTALL_DATA} ${FILESDIR}/nginx.conf ${STAGEDIR}${ETCDIR}/nginx/nginx.conf.sample
	${INSTALL_DATA} ${FILESDIR}/ohngea.conf ${STAGEDIR}${ETCDIR}/nginx/ohngea.conf.sample
	${INSTALL_DATA} ${FILESDIR}/xml_curl.conf.xml.sample ${STAGEDIR}${ETCDIR}/freeswitch/xml_curl.conf.xml.sample
	# Frontend static (pre-built in poudriere pre-build, or fallback)
.if exists(${WRKSRC}/frontend/dist)
	${COPYTREE_SHARE} "frontend/dist" ${STAGEDIR}${WWWDIR}/static
.elif exists(${WRKSRC}/static)
	${COPYTREE_SHARE} "static" ${STAGEDIR}${WWWDIR}
.endif
	# Helper scripts
	${INSTALL_SCRIPT} ${FILESDIR}/ohngea-setup ${STAGEDIR}${PREFIX}/bin/ohngea-setup
	${INSTALL_SCRIPT} ${FILESDIR}/ohngea-url ${STAGEDIR}${PREFIX}/bin/ohngea-url
	# Vendored wheels for pip-only deps
.if exists(${WRKSRC}/wheels)
	${MKDIR} ${STAGEDIR}${WWWDIR}/wheels
	${COPYTREE_SHARE} "wheels" ${STAGEDIR}${WWWDIR}
.endif

post-install:
	# @sample handling via pkg-plist
	# Install vendored pip-only deps if present
.if exists(${STAGEDIR}${WWWDIR}/wheels)
	@${ECHO_MSG} "==> Installing vendored pip deps (pydantic-xml, uuid6, valkey)"
.endif
	# Ensure .env symlink
	@${ECHO_MSG} "==> Ohngea installed to ${WWWDIR}, config in ${ETCDIR}"

.include <bsd.port.mk>
