local function has_tag(definition, wanted)
    for _, tag in ipairs(definition.tags) do
        if tag == wanted then return true end
    end
    return false
end

local card = {
    api_version = 1,
    id = "GVG_035",
    name = "Malorne",
    text = "<b>Deathrattle:</b> Go <b>Dormant</b>. Revive after 2 friendly Beasts die.",
    set = "GVG",
    type = "minion",
    class = "druid",
    rarity = "legendary",
    cost = 7,
    attack = 9,
    health = 7,
    tags = { "beast" },
    keywords = { "deathrattle" },
}

function card.on_deathrattle(ctx, self)
    ctx:move(self, "board")
    ctx:continue_with("enter_dormant")
end

function card.enter_dormant(ctx, self)
    if ctx:entity(self).zone ~= "board" then return end
    ctx:set_data(self, "malorne_dormant", 1)
    ctx:set_data(self, "beast_deaths", 0)
    cardlib.effects.grant_keyword(ctx, self, "dormant")
end

card.triggers = {
    {
        event = "entity_died",
        timing = "after",
        active_zones = { "board" },
        condition = function(ctx, self, event)
            return ctx:get_data(self, "malorne_dormant") == 1
                and event.player == ctx:controller(self)
                and event.entity ~= self
                and has_tag(ctx:card_definition(ctx:entity(event.entity).card_id), "beast")
        end,
        effect = function(ctx, self)
            local deaths = ctx:get_data(self, "beast_deaths") + 1
            ctx:set_data(self, "beast_deaths", deaths)
            if deaths >= 2 then ctx:continue_with("revive") end
        end,
    },
}

function card.revive(ctx, self)
    if ctx:get_data(self, "malorne_dormant") ~= 1 then return end
    ctx:set_data(self, "malorne_dormant", 0)
    ctx:disable_keyword(self, "dormant")
end

return card
