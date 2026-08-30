local adaptations = {
    "UNG_999t2", "UNG_999t3", "UNG_999t4", "UNG_999t5", "UNG_999t6",
    "UNG_999t7", "UNG_999t8", "UNG_999t10", "UNG_999t13", "UNG_999t14",
}

local function apply_adaptation(ctx, target, adaptation)
    if ctx:entity(target).zone ~= "board" then return end
    for _, keyword in ipairs(ctx:entity(target).keywords) do if keyword == "dormant" then return end end
    if adaptation == "UNG_999t2" then
        ctx:attach_hook(target, "on_deathrattle", "UNG_999t2")
        cardlib.effects.grant_keyword(ctx, target, "deathrattle")
    elseif adaptation == "UNG_999t3" then
        cardlib.effects.buff(ctx, target, 3, 0)
    elseif adaptation == "UNG_999t4" then
        cardlib.effects.buff(ctx, target, 0, 3)
    elseif adaptation == "UNG_999t5" then
        cardlib.effects.grant_keyword(ctx, target, "elusive")
    elseif adaptation == "UNG_999t6" then
        cardlib.effects.grant_keyword(ctx, target, "taunt")
    elseif adaptation == "UNG_999t7" then
        cardlib.effects.grant_keyword(ctx, target, "windfury")
    elseif adaptation == "UNG_999t8" then
        cardlib.effects.grant_keyword(ctx, target, "divine_shield")
    elseif adaptation == "UNG_999t10" then
        ctx:grant_keyword_until_next_turn(target, "stealth")
    elseif adaptation == "UNG_999t13" then
        cardlib.effects.grant_keyword(ctx, target, "poisonous")
    elseif adaptation == "UNG_999t14" then
        cardlib.effects.buff(ctx, target, 1, 1)
    end
end

local function request_adaptation(ctx, self)
    ctx:discover_cards(
        ctx:controller(self),
        "Adapt Galvadon",
        adaptations,
        3,
        "adapt_galvadon"
    )
end

local card = {
    api_version = 1,
    id = "UNG_954",
    name = "The Last Kaleidosaur",
    text = "<b>Quest:</b> Cast 5 spells\non your minions.\n<b>Reward:</b> Galvadon.",
    set = "UNGORO",
    type = "spell",
    class = "paladin",
    rarity = "legendary",
    cost = 1,
    keywords = { "quest" },
}

card.triggers = {
    {
        event = "spell_targeted",
        timing = "after",
        active_zones = { "secret" },
        condition = function(ctx, self, event)
            if event.player ~= ctx:controller(self) or not event.player_cast then return false end
            local target = ctx:entity(event.target)
            return target.type == "minion"
                and target.controller == ctx:controller(self)
                and ctx:get_data(self, "completed") == 0
        end,
        effect = function(ctx, self)
            local progress = ctx:get_data(self, "quest_progress") + 1
            ctx:set_data(self, "quest_progress", progress)
            if progress >= 5 then
                ctx:set_data(self, "completed", 1)
                ctx:reveal_secret(self)
                cardlib.effects.give_card(ctx, ctx:controller(self), "UNG_954t1")
            end
        end,
    },
}

card.tokens = {
    {
        id = "UNG_954t1",
        name = "Galvadon",
        text = "<b>Battlecry:</b> <b>Adapt</b> 5 times.",
        set = "UNGORO",
        type = "minion",
        class = "paladin",
        cost = 5,
        attack = 8,
        health = 8,
        tags = { "beast" },
        keywords = { "battlecry" },
        on_battlecry = function(ctx, self)
            ctx:set_data(self, "galvadon_adapts_remaining", 5)
            request_adaptation(ctx, self)
        end,
        adapt_galvadon = function(ctx, self, adaptation)
            apply_adaptation(ctx, self, adaptation)
            local remaining = ctx:get_data(self, "galvadon_adapts_remaining") - 1
            ctx:set_data(self, "galvadon_adapts_remaining", remaining)
            if remaining > 0 then request_adaptation(ctx, self) end
        end,
    },
}

return card
