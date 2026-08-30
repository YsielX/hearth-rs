local KEY = "frozen_solid"
local EXPIRY = "frozen_solid_expiry"

local card = {
    api_version = 1,
    id = "TTN_744", spell_school = "frost",
    name = "Frozen Over",
    text = "[x]Both players draw 2\ncards. Your opponent can\nnot play them next turn.",
    set = "TITANS",
    type = "spell",
    class = "death_knight",
    rarity = "rare",
    cost = 2,
    rune_cost = { frost = 1 },
}

function card.on_play(ctx, self)
    local player = ctx:controller(self)
    ctx:draw(player, 2)
    ctx:draw(ctx:opponent(player), 2)
end

card.triggers = {{
    event = "card_drawn",
    timing = "after",
    active_zones = { "graveyard" },
    condition = function(ctx, self, event)
        return event.source == self
            and event.player == ctx:opponent(ctx:controller(self))
    end,
    effect = function(ctx, self, event)
        ctx:set_data(event.entity, EXPIRY, ctx:turn() + 1)
        ctx:grant_keyword(event.entity, KEY)
    end,
}}

return card
