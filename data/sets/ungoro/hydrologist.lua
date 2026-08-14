local card = {
    api_version = 1, id = "UNG_011", name = "Hydrologist",
    text = "<b>Battlecry:</b> <b>Discover</b> and cast a <b>Secret</b>.",
    set = "UNGORO", type = "minion", class = "paladin", rarity = "common",
    cost = 2, attack = 2, health = 2, tags = { "murloc" }, keywords = { "battlecry" },
}
local function secret(definition)
    for _, keyword in ipairs(definition.keywords or {}) do if keyword == "secret" then return true end end
    return definition.secret == true
end
function card.on_battlecry(ctx, self)
    local pool = {}
    for _, id in ipairs(ctx:collectible_cards()) do
        local definition = ctx:card_definition(id)
        if definition.class == "paladin" and definition.type == "spell" and secret(definition) then
            pool[#pool + 1] = id
        end
    end
    if #pool > 0 then ctx:discover_cards(ctx:controller(self), "Choose a Secret to cast", pool, 3, "cast_hydrologist_secret") end
end
function card.cast_hydrologist_secret(ctx, self, id)
    ctx:cast_spell(ctx:controller(self), id, { skip_if_invalid = true })
end
return card
