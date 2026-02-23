package client

import (
	"encoding/json"
	"errors"
	"fmt"
	"net/http"
	"net/url"

	"github.com/rs/zerolog/log"
)

func (p *PcClient) startNamespace(name string) error {
	url := fmt.Sprintf("http://%s/namespace/start/%s", p.address, url.PathEscape(name))
	resp, err := p.client.Post(url, "application/json", nil)
	if err != nil {
		return err
	}
	defer resp.Body.Close()
	if resp.StatusCode == http.StatusOK {
		return nil
	}

	var respErr pcError
	if err = json.NewDecoder(resp.Body).Decode(&respErr); err != nil {
		log.Error().Msgf("failed to decode start namespace %s response: %v", name, err)
		return err
	}
	return errors.New(respErr.Error)
}

func (p *PcClient) stopNamespace(name string) (map[string]string, error) {
	url := fmt.Sprintf("http://%s/namespace/stop/%s", p.address, url.PathEscape(name))
	return p.namespacePatch(url, "failed to stop some processes")
}

func (p *PcClient) restartNamespace(name string) error {
	url := fmt.Sprintf("http://%s/namespace/restart/%s", p.address, url.PathEscape(name))
	resp, err := p.client.Post(url, "application/json", nil)
	if err != nil {
		return err
	}
	defer resp.Body.Close()
	if resp.StatusCode == http.StatusOK {
		return nil
	}

	var respErr pcError
	if err = json.NewDecoder(resp.Body).Decode(&respErr); err != nil {
		log.Error().Msgf("failed to decode restart namespace %s response: %v", name, err)
		return err
	}
	return errors.New(respErr.Error)
}

func (p *PcClient) getNamespaces() ([]string, error) {
	url := fmt.Sprintf("http://%s/namespaces", p.address)
	resp, err := p.client.Get(url)
	if err != nil {
		return nil, err
	}
	defer resp.Body.Close()

	if resp.StatusCode != http.StatusOK {
		var respErr pcError
		if err = json.NewDecoder(resp.Body).Decode(&respErr); err != nil {
			log.Error().Msgf("failed to decode get namespaces response: %v", err)
			return nil, err
		}
		return nil, errors.New(respErr.Error)
	}

	var namespaces []string
	if err = json.NewDecoder(resp.Body).Decode(&namespaces); err != nil {
		return nil, err
	}
	return namespaces, nil
}

// namespacePatch is a helper to perform PATCH actions on namespace endpoints
// and unify response decoding and error handling.
func (p *PcClient) namespacePatch(url string, partialErrMsg string) (map[string]string, error) {
	return p.doMapRequest(http.MethodPatch, url, nil, partialErrMsg)
}

func (p *PcClient) DisableNamespace(name string) (map[string]string, error) {
	url := fmt.Sprintf("http://%s/namespace/disable/%s", p.address, name)
	return p.namespacePatch(url, "failed to disable some processes")
}

func (p *PcClient) EnableNamespace(name string) (map[string]string, error) {
	url := fmt.Sprintf("http://%s/namespace/enable/%s", p.address, name)
	return p.namespacePatch(url, "failed to enable some processes")
}

func (p *PcClient) RemoveNamespace(name string) (map[string]string, error) {
	url := fmt.Sprintf("http://%s/namespace?name=%s", p.address, name)
	return p.doMapRequest(http.MethodDelete, url, nil, "failed to remove some processes")
}
