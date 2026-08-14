local tribes = { "beast", "dragon", "murloc" }

local function has_tribe(ctx, entity, tribe)
    for _, tag in ipairs(ctx:card_definition(ctx:entity(entity).card_id).tags or {}) do
        if tag == tribe or tag == "all" then return true end
    end
    return false
end

local function choose_next_draw(ctx, self, stage)
    if stage > #tribes then return end
    local candidates = {}
    for _, entity in ipairs(ctx:deck(ctx:controller(self))) do
        if has_tribe(ctx, entity, tribes[stage]) then
            candidates[#candidates + 1] = entity
        end
    end
    if #candidates == 0 then
        ctx:continue_with_value("continue_curator_draws", stage + 1)
        return
    end
    ctx:set_data(self, "curator_stage", stage)
    ctx:random_entity(candidates, "draw_curator_minion")
end

local card = {
    api_version = 1,
    id = "KAR_061",
    name = "The Curator",
    text = "<b>Taunt</b>\n<b>Battlecry:</b> Draw a Beast, Dragon, and Murloc.",
    set = "KARA",
    type = "minion",
    rarity = "legendary",
    tags = { "mech" },
    cost = 5,
    attack = 4,
    health = 6,
    keywords = { "taunt", "battlecry" },
}

function card.on_battlecry(ctx, self)
    ctx:continue_with_value("continue_curator_draws", 1)
end

function card.continue_curator_draws(ctx, self, stage)
    choose_next_draw(ctx, self, stage)
end

function card.draw_curator_minion(ctx, self, entity)
    local stage = ctx:get_data(self, "curator_stage") or 1
    ctx:draw_entity(ctx:controller(self), entity)
    ctx:continue_with_value("continue_curator_draws", stage + 1)
end

return card
