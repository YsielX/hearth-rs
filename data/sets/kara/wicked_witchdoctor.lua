local basic_totems = { "NEW1_009", "CS2_050", "CS2_051", "CS2_052" }

local card = {
    api_version = 1,
    id = "KAR_021",
    name = "Wicked Witchdoctor",
    text = "Whenever you cast a spell, summon a random basic Totem.",
    set = "KARA",
    type = "minion",
    class = "shaman",
    rarity = "common",
    cost = 3,
    attack = 3,
    health = 4,
    triggers = {{
        event = "spell_cast",
        timing = "after",
        active_zones = { "board" },
        condition = function(ctx, self, event)
            return event.player == ctx:controller(self) and event.player_cast
        end,
        effect = function(ctx)
            ctx:continue_with("choose_basic_totem")
        end,
    }},
}

function card.choose_basic_totem(ctx, self)
    if #ctx:board(ctx:controller(self)) < 7 then
        ctx:random_value(basic_totems, "summon_basic_totem")
    end
end

function card.summon_basic_totem(ctx, self, card_id)
    ctx:summon(ctx:controller(self), card_id)
end

return card
