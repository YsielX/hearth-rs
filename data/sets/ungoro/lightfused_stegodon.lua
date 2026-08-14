local adaptations = {
    "UNG_999t2", "UNG_999t3", "UNG_999t4", "UNG_999t5", "UNG_999t6",
    "UNG_999t7", "UNG_999t8", "UNG_999t10", "UNG_999t13", "UNG_999t14",
}

local function apply_adaptation(ctx, target, adaptation)
    if ctx:entity(target).zone ~= "board" then return end
    for _, keyword in ipairs(ctx:entity(target).keywords) do if keyword == "dormant" then return end end
    if adaptation == "UNG_999t2" then
        ctx:attach_deathrattle(target, "UNG_999t2")
        ctx:grant_keyword(target, "deathrattle")
    elseif adaptation == "UNG_999t3" then
        ctx:buff(target, 3, 0)
    elseif adaptation == "UNG_999t4" then
        ctx:buff(target, 0, 3)
    elseif adaptation == "UNG_999t5" then
        ctx:grant_keyword(target, "elusive")
    elseif adaptation == "UNG_999t6" then
        ctx:grant_keyword(target, "taunt")
    elseif adaptation == "UNG_999t7" then
        ctx:grant_keyword(target, "windfury")
    elseif adaptation == "UNG_999t8" then
        ctx:grant_keyword(target, "divine_shield")
    elseif adaptation == "UNG_999t10" then
        ctx:grant_keyword_until_next_turn(target, "stealth")
    elseif adaptation == "UNG_999t13" then
        ctx:grant_keyword(target, "poisonous")
    elseif adaptation == "UNG_999t14" then
        ctx:buff(target, 1, 1)
    end
end

local card = {
    api_version = 1,
    id = "UNG_962",
    name = "Lightfused Stegodon",
    text = "<b>Battlecry:</b> <b>Adapt</b> your Silver Hand Recruits.",
    set = "UNGORO",
    type = "minion",
    class = "paladin",
    rarity = "rare",
    cost = 3,
    attack = 3,
    health = 3,
    tags = { "beast" },
    keywords = { "battlecry" },
}

function card.on_battlecry(ctx, self)
    local recruits = {}
    for _, minion in ipairs(ctx:friendly_minions(self)) do
        if ctx:entity(minion).card_id == "CS2_101t" then
            recruits[#recruits + 1] = minion
        end
    end
    if #recruits == 0 then return end

    ctx:set_data(self, "recruit_count", #recruits)
    for index, recruit in ipairs(recruits) do
        ctx:set_data(self, "recruit_" .. index, recruit)
    end
    ctx:discover_cards(
        ctx:controller(self),
        "Adapt your Silver Hand Recruits",
        adaptations,
        3,
        "adapt_recruits"
    )
end

function card.adapt_recruits(ctx, self, adaptation)
    local player = ctx:controller(self)
    for index = 1, ctx:get_data(self, "recruit_count") do
        local recruit = ctx:get_data(self, "recruit_" .. index)
        local entity = ctx:entity(recruit)
        if entity.zone == "board" and entity.controller == player
            and entity.card_id == "CS2_101t"
        then
            apply_adaptation(ctx, recruit, adaptation)
        end
    end
end

return card
