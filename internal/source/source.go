package source

import (
	"bytes"
	"errors"
	"io/ioutil"
	"mime/multipart"
	"net/http"

	"github.com/faldez/tanoshi/internal/lua/helper"
	"github.com/faldez/tanoshi/internal/lua/scraper"
	lua "github.com/yuin/gopher-lua"
	luajson "layeh.com/gopher-json"
	luar "layeh.com/gopher-luar"
)

type Source struct {
	Name       string
	URL        string
	l          *lua.LState
	httpClient *http.Client
	header     http.Header
}

// LoadSourceFromPath load source from specified path
func LoadSourceFromPath(path string) (*Source, error) {
	s := newSource()

	s.httpClient = &http.Client{}
	s.header = make(http.Header)

	s.l.PreloadModule("scraper", scraper.NewHTMLScraper().Loader)
	s.l.PreloadModule("helper", helper.NewHelper().Loader)
	luajson.Preload(s.l)

	s.l.SetGlobal(luaMangaTypeName, luar.NewType(s.l, Manga{}))
	s.l.SetGlobal(luaChapterTypeName, luar.NewType(s.l, Chapter{}))
	s.l.SetGlobal(luaPageTypeName, luar.NewType(s.l, Page{}))

	if err := s.l.DoFile(path); err != nil {
		return nil, err
	}

	if err := s.getName(); err != nil {
		s.l.Close()
		return nil, err
	}
	if err := s.getBaseURL(); err != nil {
		s.l.Close()
		return nil, err
	}

	return s, nil
}

func newSource() *Source {
	return &Source{l: lua.NewState()}
}

func (s *Source) getName() error {
	if err := s.callLuaFunc("name"); err != nil {
		return err
	}
	s.Name = s.l.CheckString(1)
	s.l.Pop(1)
	return nil
}

func (s *Source) getBaseURL() error {
	if err := s.callLuaFunc("base_url"); err != nil {
		return err
	}
	s.URL = s.l.CheckString(1)
	s.l.Pop(1)
	return nil
}

func (s *Source) getLatestUpdatesRequest(page int) (*SourceResponse, error) {
	if err := s.callLuaFunc("get_latest_updates_request", lua.LNumber(page)); err != nil {
		return nil, err
	}

	request, err := s.createRequest()
	if err != nil {
		return nil, err
	}

	resp, err := s.doRequest(request)
	if err != nil {
		return nil, err
	}

	return resp, nil
}

func (s *Source) getLatestUpdates(body *string) ([]*Manga, error) {
	if err := s.callLuaFunc("get_latest_updates", lua.LString(*body)); err != nil {
		return nil, err
	}
	lv := s.l.Get(-1)

	var manga []*Manga
	if tbl, ok := lv.(*lua.LTable); ok {
		tbl.ForEach(func(_, v lua.LValue) {
			if ud, ok := v.(*lua.LUserData); ok {
				if m, ok := ud.Value.(*Manga); ok {
					m.Source = s.Name
					manga = append(manga, m)
				}
			}
		})
	}
	return manga, nil
}

// GetLatestUpdates get latest updates from source and return list of manga
func (s *Source) GetLatestUpdates(page int) ([]*Manga, error) {
	res, err := s.getLatestUpdatesRequest(page)
	if err != nil {
		return nil, err
	}

	mangaList, err := s.getLatestUpdates(&res.Body)
	if err != nil {
		return nil, err
	}

	return mangaList, nil
}

func (s *Source) getMangaDetailsRequest(m *Manga) (*SourceResponse, error) {
	if err := s.callLuaFunc("get_manga_details_request", luar.New(s.l, *m)); err != nil {
		return nil, err
	}

	request, err := s.createRequest()
	if err != nil {
		return nil, err
	}

	resp, err := s.doRequest(request)
	if err != nil {
		return nil, err
	}

	return resp, nil
}

func (s *Source) getMangaDetails(body *string) (*Manga, error) {
	if err := s.callLuaFunc("get_manga_details", lua.LString(*body)); err != nil {
		return nil, err
	}
	lv := s.l.Get(-1)

	var manga *Manga
	ud := lv.(*lua.LUserData)
	manga = ud.Value.(*Manga)
	manga.Source = s.Name
	return manga, nil
}

// GetMangaDetails get details for a manga
func (s *Source) GetMangaDetails(m *Manga) (*Manga, error) {
	res, err := s.getMangaDetailsRequest(m)
	if err != nil {
		return nil, err
	}
	manga, err := s.getMangaDetails(&res.Body)
	if err != nil {
		return nil, err
	}
	manga.ID = m.ID

	return manga, nil
}

func (s *Source) getChaptersRequest(m *Manga) (*SourceResponse, error) {
	if err := s.callLuaFunc("get_chapters_request", luar.New(s.l, *m)); err != nil {
		return nil, err
	}

	req, err := s.createRequest()
	if err != nil {
		return nil, err
	}

	resp, err := s.doRequest(req)
	if err != nil {
		return nil, err
	}

	return resp, nil
}

func (s *Source) getChapters(body *string) ([]*Chapter, error) {
	if err := s.callLuaFunc("get_chapters", lua.LString(*body)); err != nil {
		return nil, err
	}
	lv := s.l.Get(-1)

	var chapters []*Chapter
	if tbl, ok := lv.(*lua.LTable); ok {
		tbl.ForEach(func(_, v lua.LValue) {
			if ud, ok := v.(*lua.LUserData); ok {
				if c, ok := ud.Value.(*Chapter); ok {
					c.Source = s.Name
					chapters = append(chapters, c)
				}
			}
		})
	}
	return chapters, nil
}

// GetChapters get list of chapter of a manga
func (s *Source) GetChapters(m *Manga) ([]*Chapter, error) {
	res, err := s.getChaptersRequest(m)
	if err != nil {
		return nil, err
	}
	chapters, err := s.getChapters(&res.Body)
	if err != nil {
		return nil, err
	}

	return chapters, nil
}

func (s *Source) getChapterRequest(c *Chapter) (*SourceResponse, error) {
	if err := s.callLuaFunc("get_chapter_request", luar.New(s.l, *c)); err != nil {
		return nil, err
	}

	req, err := s.createRequest()
	if err != nil {
		return nil, err
	}

	resp, err := s.doRequest(req)
	if err != nil {
		return nil, err
	}

	return resp, nil
}

func (s *Source) getChapter(body *string) (*Chapter, error) {
	if err := s.callLuaFunc("get_chapter", lua.LString(*body)); err != nil {
		return nil, err
	}
	lv := s.l.Get(-1)

	var chapter *Chapter
	ud := lv.(*lua.LUserData)
	chapter = ud.Value.(*Chapter)

	return chapter, nil
}

// GetChapter get detail from a chapter
func (s *Source) GetChapter(c *Chapter) (*Chapter, error) {
	res, err := s.getChapterRequest(c)
	if err != nil {
		return nil, err
	}
	chapter, err := s.getChapter(&res.Body)
	if err != nil {
		return nil, err
	}
	chapter.ID = c.ID

	return chapter, nil
}

func (s *Source) loginRequest(username, password, twoFactor string, remember bool) (*SourceResponse, error) {
	param := map[string]string{
		"username":    username,
		"password":    password,
		"two_factor":  twoFactor,
		"remember_me": "1",
	}
	if err := s.callLuaFunc("login_request", luar.New(s.l, param)); err != nil {
		return nil, err
	}

	req, err := s.createRequest()
	if err != nil {
		return nil, err
	}

	resp, err := s.doRequest(req)
	if err != nil {
		return nil, err
	}

	return resp, nil
}

func (s *Source) login(resp *SourceResponse) error {
	header := map[string][]string(resp.Header)
	body := resp.Body
	if err := s.callLuaFunc("login", luar.New(s.l, header), lua.LString(body)); err != nil {
		return err
	}
	lv := s.l.Get(-1)
	if tbl, ok := lv.(*lua.LTable); ok {
		tbl.ForEach(func(k, v lua.LValue) {
			if values, ok := v.(*lua.LTable); ok {
				s.header.Del(k.String())
				values.ForEach(func(i, w lua.LValue) {
					s.header.Add(k.String(), w.String())
				})
			}
		})
	} else {
		return errors.New("Table expected")
	}
	return nil
}

// Login login to source
func (s *Source) Login(username, password, twoFactor string, remember bool) error {
	resp, err := s.loginRequest(username, password, twoFactor, remember)
	if err != nil {
		return err
	}

	err = s.login(resp)
	if err != nil {
		return err
	}

	return nil
}

func (s *Source) fetchMangaRequest(filter Filters) (*SourceResponse, error) {
	if err := s.callLuaFunc("fetch_manga_request", filter.ToLTable()); err != nil {
		return nil, err
	}

	req, err := s.createRequest()
	if err != nil {
		return nil, err
	}

	resp, err := s.doRequest(req)
	if err != nil {
		return nil, err
	}

	return resp, nil
}

func (s *Source) fetchManga(body *string) ([]*Manga, error) {
	if err := s.callLuaFunc("fetch_manga", lua.LString(*body)); err != nil {
		return nil, err
	}
	lv := s.l.Get(-1)

	var manga []*Manga
	if tbl, ok := lv.(*lua.LTable); ok {
		tbl.ForEach(func(_, v lua.LValue) {
			if ud, ok := v.(*lua.LUserData); ok {
				if m, ok := ud.Value.(*Manga); ok {
					manga = append(manga, m)
				}
			}
		})
	}
	return manga, nil
}

func (s *Source) FetchManga(filter Filters) ([]*Manga, error) {
	res, err := s.fetchMangaRequest(filter)
	if err != nil {
		return nil, err
	}

	mangaList, err := s.fetchManga(&res.Body)
	if err != nil {
		return nil, err
	}

	return mangaList, nil
}

func (s *Source) headerBuilder() *http.Header {
	var header http.Header
	if s.header != nil {
		header = s.header.Clone()
	} else {
		header = make(http.Header)
	}

	header.Set("User-Agent", "Tanoshi/0.1.0")
	return &header
}

func (s *Source) createRequest() (*http.Request, error) {
	lv := s.l.Get(-1)

	req, ok := lv.(*lua.LTable)
	if !ok {
		return nil, errors.New("table expected")
	}

	var (
		buffer    bytes.Buffer
		headerMap *http.Header = s.headerBuilder()
	)

	headers, headersOk := req.RawGetString("header").(*lua.LTable)
	if headersOk {
		headers.ForEach(func(k lua.LValue, v lua.LValue) {
			headerMap.Set(k.String(), v.String())
		})
	}

	contentType := headerMap.Get("Content-Type")
	data, dataOk := req.RawGetString("data").(*lua.LTable)
	if dataOk {
		switch contentType {
		case "multipart/form-data":
			writer := multipart.NewWriter(&buffer)

			data.ForEach(func(k lua.LValue, v lua.LValue) {
				writer.WriteField(k.String(), v.String())
			})

			writer.Close()
			headerMap.Set("Content-Type", writer.FormDataContentType())
			break
		}
	}

	method := req.RawGetString("method").String()
	url := req.RawGetString("url").String()

	request, err := http.NewRequest(method, url, &buffer)
	if err != nil {
		return nil, err
	}
	request.Header = *headerMap

	return request, nil
}

func (s *Source) doRequest(req *http.Request) (*SourceResponse, error) {
	resp, err := s.httpClient.Do(req)
	if err != nil {
		return nil, err
	}

	body, err := ioutil.ReadAll(resp.Body)
	if err != nil {
		return nil, err
	}

	response := SourceResponse{
		Header: resp.Header,
		Body:   string(body),
	}

	return &response, nil
}

func (s *Source) callLuaFunc(name string, args ...lua.LValue) error {
	return s.l.CallByParam(lua.P{
		Fn:      s.l.GetGlobal(name),
		NRet:    1,
		Protect: true,
	}, args...)
}
